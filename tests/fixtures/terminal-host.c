// Stand in for Zed's executable while retaining its PID as the bridge's ancestor.
#include <sys/wait.h>
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc < 2) return 1;
    pid_t child = fork();
    if (child < 0) return 1;
    if (child == 0) {
        execv(argv[1], argv + 1);
        _exit(127);
    }
    int status;
    if (waitpid(child, &status, 0) < 0) return 1;
    return WIFEXITED(status) ? WEXITSTATUS(status) : 1;
}
