// A palette input method observes geometry without handling or changing input.
#import <AppKit/AppKit.h>
#import <Carbon/Carbon.h>
#import <InputMethodKit/InputMethodKit.h>
#import <ApplicationServices/ApplicationServices.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <unistd.h>

static NSString *const BundleID = @"dev.figfig.cursor";
static NSString *const RequestName = @"dev.figfig.cursor.request";

static NSDictionary *HostProfile(NSString *identifier) {
    static NSArray *hosts;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
        NSData *data = [[NSString stringWithUTF8String:FIGFIG_HOSTS_JSON] dataUsingEncoding:NSUTF8StringEncoding];
        hosts = [NSJSONSerialization JSONObjectWithData:data options:0 error:NULL];
    });
    for (NSDictionary *host in hosts) {
        if ([host[@"backend"] isEqualToString:@"inputMethod"] &&
            [host[@"bundleId"] isEqualToString:identifier]) return host;
    }
    return nil;
}

static NSRect CursorRect(id<IMKTextInput> client, NSDictionary *profile) {
    NSRect rect = NSZeroRect;
    NSRange selected = [client selectedRange];
    NSRange marked = [client markedRange];
    // Only ask for geometry of an unselected terminal insertion point.
    BOOL isZed = [profile[@"family"] isEqualToString:@"zed"];
    if (selected.location != NSNotFound && (!isZed || selected.location == 0) &&
        selected.length == 0 && marked.length == 0) {
        [client attributesForCharacterIndex:selected.location lineHeightRectangle:&rect];
    }
    if (!isfinite(rect.origin.x) || !isfinite(rect.origin.y) ||
        !isfinite(rect.size.height) || rect.size.height < 1 || rect.size.height > 200) return NSZeroRect;
    return rect;
}

// Chromium exposes xterm's input class through AX. Do not accept an editor or
// search field just because it belongs to the same application/input client.
static BOOL IsTerminalClasses(NSArray *classes) {
    return [classes isKindOfClass:NSArray.class] && [classes containsObject:@"xterm-helper-textarea"];
}

static uint64_t TerminalFocus(pid_t pid, NSDictionary *profile) {
    if (![profile[@"family"] isEqualToString:@"electron"]) return 0;
    AXUIElementRef application = AXUIElementCreateApplication(pid);
    AXUIElementSetMessagingTimeout(application, 0.05);
    CFTypeRef element = NULL;
    AXError result = AXUIElementCopyAttributeValue(application, kAXFocusedUIElementAttribute, &element);
    CFRelease(application);
    if (result != kAXErrorSuccess || !element) return 0;
    CFTypeRef classes = NULL;
    AXUIElementCopyAttributeValue((AXUIElementRef)element, CFSTR("AXDOMClassList"), &classes);
    uint64_t focus = IsTerminalClasses((__bridge NSArray *)classes) ? (uint64_t)CFHash(element) : 0;
    if (classes) CFRelease(classes);
    CFRelease(element);
    return focus;
}

static void SendPosition(NSString *path, NSDictionary *position) {
    // Only connect inside a private, user-owned runtime directory.
    struct stat directory;
    if (lstat(path.stringByDeletingLastPathComponent.fileSystemRepresentation, &directory) != 0 ||
        !S_ISDIR(directory.st_mode) || directory.st_uid != getuid() || (directory.st_mode & 0077)) return;
    struct stat socketInfo;
    if (![path.lastPathComponent isEqualToString:@"app.sock"] ||
        lstat(path.fileSystemRepresentation, &socketInfo) != 0 || !S_ISSOCK(socketInfo.st_mode) ||
        socketInfo.st_uid != getuid() || (socketInfo.st_mode & 0077)) return;
    struct sockaddr_un address = { .sun_family = AF_UNIX };
    const char *socketPath = path.fileSystemRepresentation;
    if (strlen(socketPath) >= sizeof(address.sun_path)) return;
    strlcpy(address.sun_path, socketPath, sizeof(address.sun_path));
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return;
    int noSignal = 1;
    setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &noSignal, sizeof(noSignal));
    struct timeval timeout = { .tv_sec = 0, .tv_usec = 250000 };
    setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof(timeout));
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));
    if (connect(fd, (struct sockaddr *)&address, sizeof(address)) == 0) {
        // Consume the service's initial preferences before sending a one-way report.
        char preferences[4096];
        if (recv(fd, preferences, sizeof(preferences), 0) > 0) {
            NSDictionary *message = @{ @"type": @"cursorPosition", @"position": position };
            NSMutableData *data = [[NSJSONSerialization dataWithJSONObject:message options:0 error:NULL] mutableCopy];
            [data appendBytes:"\n" length:1];
            const char *bytes = data.bytes;
            NSUInteger remaining = data.length;
            while (remaining > 0) {
                ssize_t written = send(fd, bytes, remaining, 0);
                if (written <= 0) break;
                bytes += written;
                remaining -= written;
            }
        }
    }
    close(fd);
}

@interface FigfigCursorController : IMKInputController
@property BOOL active;
@property uint64_t focusID;
@end

@implementation FigfigCursorController
- (instancetype)initWithServer:(IMKServer *)server delegate:(id)delegate client:(id)client {
    self = [super initWithServer:server delegate:delegate client:client];
    if (self) {
        self.active = YES;
        self.focusID = ((uint64_t)arc4random() << 32) | arc4random();
        if (self.focusID == 0) self.focusID = 1;
        [[NSDistributedNotificationCenter defaultCenter] addObserver:self selector:@selector(reportPosition:)
            name:RequestName object:nil suspensionBehavior:NSNotificationSuspensionBehaviorDeliverImmediately];
    }
    return self;
}
- (void)dealloc {
    [[NSDistributedNotificationCenter defaultCenter] removeObserver:self];
}
- (NSUInteger)recognizedEvents:(id)sender { return 0; }
- (void)activateServer:(id)sender { self.active = YES; }
- (void)deactivateServer:(id)sender { self.active = NO; }
- (void)reportPosition:(NSNotification *)notification {
    NSDictionary *request = notification.userInfo;
    NSString *path = request[@"socketPath"];
    NSString *requestID = request[@"requestId"];
    NSString *pidString = request[@"terminalPid"];
    if (!self.active || ![path isKindOfClass:NSString.class] || ![requestID isKindOfClass:NSString.class] ||
        ![pidString isKindOfClass:NSString.class] || requestID.length > 256) return;
    NSRunningApplication *front = NSWorkspace.sharedWorkspace.frontmostApplication;
    id<IMKTextInput> client = (id<IMKTextInput>)[self client];
    NSDictionary *profile = HostProfile(front.bundleIdentifier);
    NSDictionary *clientProfile = HostProfile([client bundleIdentifier]);
    if (!profile || !clientProfile || ![profile[@"bundleId"] isEqual:clientProfile[@"bundleId"]] ||
        front.processIdentifier != pidString.intValue) return;
    // JetBrains supplies precise geometry and terminal focus from its adapter.
    if ([profile[@"family"] isEqualToString:@"jetbrains"]) return;
    uint64_t focus = self.focusID;
    if ([profile[@"family"] isEqualToString:@"electron"]) {
        focus = TerminalFocus(front.processIdentifier, profile);
        if (focus == 0) return;
    }
    NSRect rect = CursorRect(client, profile);
    NSDictionary *position = @{ @"requestId": requestID, @"terminalPid": @(front.processIdentifier),
        @"x": @(rect.origin.x), @"y": @(rect.origin.y), @"height": @(rect.size.height), @"focus": @(focus) };
    dispatch_async(dispatch_get_global_queue(QOS_CLASS_USER_INTERACTIVE, 0), ^{ SendPosition(path, position); });
}
@end

int main(void) {
    @autoreleasepool {
        NSApplication *app = NSApplication.sharedApplication;
        IMKServer *server = [[IMKServer alloc] initWithName:@"dev.figfig.cursor.connection" bundleIdentifier:BundleID];
        if (!server) return 1;
        [[NSDistributedNotificationCenter defaultCenter] addObserverForName:@"dev.figfig.cursor.stop" object:nil
            queue:NSOperationQueue.mainQueue usingBlock:^(NSNotification *notification) { [app terminate:nil]; }];
        [app run];
        (void)server;
    }
    return 0;
}
