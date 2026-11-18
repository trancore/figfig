// Exercise the production tracker without registering an input source or reading any app.
#define main FigfigCursorMain
#include "../../macos/cursor/main.m"
#undef main

@interface FixtureClient : NSObject
@property NSRange selection;
@property NSRange marked;
@property NSRect geometry;
@property NSUInteger geometryQueries;
@end
@implementation FixtureClient
- (NSRange)selectedRange { return self.selection; }
- (NSRange)markedRange { return self.marked; }
- (NSDictionary *)attributesForCharacterIndex:(NSUInteger)index lineHeightRectangle:(NSRect *)rect {
    self.geometryQueries++;
    *rect = self.geometry;
    return @{};
}
@end

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 2) return 1;
        FixtureClient *client = [FixtureClient new];
        client.geometry = NSMakeRect(-1500, 800, 1, 20);
        NSRect rect = CursorRect((id<IMKTextInput>)client, HostProfile(@"dev.zed.Zed"));
        if (!NSEqualRects(rect, client.geometry) || client.geometryQueries != 1) return 2;
        client.selection = NSMakeRange(1, 0);
        if (!NSEqualRects(CursorRect((id<IMKTextInput>)client, HostProfile(@"dev.zed.Zed")), NSZeroRect)) return 3;
        client.selection = NSMakeRange(0, 2);
        if (!NSEqualRects(CursorRect((id<IMKTextInput>)client, HostProfile(@"dev.zed.Zed")), NSZeroRect)) return 4;
        client.selection = NSMakeRange(0, 0);
        client.marked = NSMakeRange(0, 2);
        if (!NSEqualRects(CursorRect((id<IMKTextInput>)client, HostProfile(@"dev.zed.Zed")), NSZeroRect)) return 5;
        if (client.geometryQueries != 1) return 6;
        client.marked = NSMakeRange(NSNotFound, 0);
        for (NSNumber *height in @[@0, @(-1), @201, @(NAN), @(INFINITY)]) {
            client.geometry = NSMakeRect(100, 200, 1, height.doubleValue);
            if (!NSEqualRects(CursorRect((id<IMKTextInput>)client, HostProfile(@"dev.zed.Zed")), NSZeroRect)) return 7;
        }
        if (!HostProfile(@"dev.zed.Zed") || !HostProfile(@"dev.zed.Zed-Preview") || HostProfile(@"com.apple.Terminal")) return 8;
        client.geometry = NSMakeRect(100, 200, 1, 20);
        client.selection = NSMakeRange(8, 0);
        if (!NSEqualRects(CursorRect((id<IMKTextInput>)client, HostProfile(@"com.microsoft.VSCode")), client.geometry)) return 9;
        client.marked = NSMakeRange(0, 1);
        if (!NSEqualRects(CursorRect((id<IMKTextInput>)client, HostProfile(@"com.microsoft.VSCode")), NSZeroRect)) return 10;
        if (!IsTerminalClasses(@[@"xterm-helper-textarea"]) || IsTerminalClasses(@[@"monaco-editor"]) || IsTerminalClasses(nil)) return 11;
        if (!HostProfile(@"com.todesktop.230313mzl4w4u92") || !HostProfile(@"com.jetbrains.intellij")) return 12;
        SendPosition([NSString stringWithUTF8String:argv[1]], @{ @"requestId": @"fixture", @"terminalPid": @42,
            @"x": @(-1500), @"y": @800, @"height": @20, @"focus": @123 });
        return 0;
    }
}
