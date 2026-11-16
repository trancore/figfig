export interface CommandSpec {
  name: string;
  description: string;
  options: OptionSpec[];
  subcommands: SubcommandSpec[];
}
export interface OptionSpec {
  name: string;
  description: string;
  takesValue?: boolean;
}
export interface SubcommandSpec {
  name: string;
  description: string;
  options: OptionSpec[];
  generator?: "gitBranches" | "packageScripts";
}
function option(name: string, description: string, takesValue = false): OptionSpec {
  return { name, description, takesValue };
}

function sub(
  name: string,
  description: string,
  options: OptionSpec[] = [],
  generator?: SubcommandSpec["generator"],
): SubcommandSpec {
  return { name, description, options, ...(generator ? { generator } : {}) };
}

const help = option("--help", "Show command help");

export const specs: CommandSpec[] = [
  {
    name: "git",
    description: "Distributed version control",
    options: [
      help,
      option("--version", "Show Git version"),
      option("-C", "Run in a directory", true)
    ],
    subcommands: [
      sub("status", "Show working tree status", [
        option("--short", "Compact output"),
        option("--branch", "Show branch information")
      ]),
      sub("add", "Stage file changes", [
        option("--all", "Stage all changes"),
        option("--patch", "Interactively stage changes")
      ]),
      sub("commit", "Record staged changes", [
        option("-m", "Commit message", true),
        option("--amend", "Amend the last commit"),
        option("--signoff", "Add Signed-off-by")
      ]),
      sub("checkout", "Switch branches or restore files", [
        option("-b", "Create a new branch", true)
      ], "gitBranches"),
      sub("switch", "Switch branches", [
        option("-c", "Create a new branch", true)
      ], "gitBranches"),
      sub("branch", "List or manage branches", [
        option("--all", "Include remote branches"),
        option("--delete", "Delete a branch")
      ], "gitBranches"),
      sub("merge", "Merge a branch", [
        option("--no-ff", "Always create a merge commit"),
        option("--abort", "Abort the merge")
      ], "gitBranches"),
      sub("rebase", "Reapply commits", [
        option("--interactive", "Edit the commit sequence"),
        option("--abort", "Abort the rebase"),
        option("--continue", "Continue the rebase")
      ], "gitBranches"),
      sub("diff", "Show changes", [
        option("--staged", "Show staged changes"),
        option("--stat", "Show a summary")
      ]),
      sub("log", "Show commit history", [
        option("--oneline", "One line per commit"),
        option("--graph", "Show the commit graph"),
        option("--all", "Include all refs")
      ]),
      sub("push", "Push commits", [
        option("--set-upstream", "Set upstream tracking"),
        option("--force-with-lease", "Require the remote ref to be unchanged")
      ]),
      sub("pull", "Fetch and integrate changes", [
        option("--rebase", "Rebase local commits"),
        option("--ff-only", "Only fast-forward")
      ]),
      sub("fetch", "Fetch remote refs", [
        option("--all", "Fetch all remotes"),
        option("--prune", "Remove deleted remote refs")
      ]),
      sub("clone", "Clone a repository", [
        option("--depth", "Limit history depth", true),
        option("--branch", "Select a branch", true)
      ]),
      sub("restore", "Restore working tree files", [
        option("--staged", "Restore the index"),
        option("--source", "Restore from a tree", true)
      ]),
      sub("stash", "Store uncommitted changes"),
      sub("remote", "Manage remote repositories"),
      sub("tag", "Manage tags"),
    ],
  },
  {
    name: "brew",
    description: "The macOS package manager",
    options: [
      help,
      option("--version", "Show Homebrew version")
    ],
    subcommands: [
      sub("install", "Install a package", [
        option("--cask", "Install a macOS app"),
        option("--formula", "Install a formula"),
        option("--build-from-source", "Build locally")
      ]),
      sub("uninstall", "Remove a package"),
      sub("upgrade", "Upgrade packages"),
      sub("update", "Update Homebrew"),
      sub("search", "Search for packages"),
      sub("info", "Show package information"),
      sub("list", "List installed packages", [
        option("--formula", "List formulae"),
        option("--cask", "List casks")
      ]),
      sub("doctor", "Check Homebrew configuration"),
      sub("tap", "Add a package repository"),
      sub("services", "Manage background services"),
      sub("cleanup", "Remove old package versions")
    ],
  },
  ...["npm", "pnpm"].map((name): CommandSpec => ({
    name,
    description: "JavaScript package manager",
    options: [
      help,
      option("--version", "Show version")
    ],
    subcommands: [
      sub("install", "Install dependencies", [
        option("--save-dev", "Add a development dependency"),
        option("--global", "Install globally"),
        option("--ignore-scripts", "Skip lifecycle scripts")
      ]),
      sub("add", "Add a dependency", [
        option("--save-dev", "Add a development dependency")
      ]),
      sub("run", "Run a package script", [], "packageScripts"),
      sub("test", "Run tests"),
      sub("start", "Start the project"),
      sub("build", "Build the project"),
      sub("exec", "Execute a package binary"),
      sub("remove", "Remove a dependency"),
      sub("update", "Update dependencies"),
      sub("outdated", "Check dependency versions"),
      sub("init", "Create a package.json"),
      sub("publish", "Publish a package"),
      ...(name === "npm" ? [
        sub("ci", "Install locked dependencies")
      ] : [])
    ],
  })),
  {
    name: "docker",
    description: "Manage containers and images",
    options: [
      help,
      option("--version", "Show Docker version"),
      option("--context", "Select a context", true)
    ],
    subcommands: [
      sub("run", "Run a container", [
        option("--detach", "Run in the background"),
        option("--rm", "Remove on exit"),
        option("--name", "Set a container name", true),
        option("-p", "Publish a port", true),
        option("-v", "Mount a volume", true),
        option("-it", "Interactive terminal")
      ]),
      sub("build", "Build an image", [
        option("-t", "Set image tag", true),
        option("-f", "Select Dockerfile", true),
        option("--no-cache", "Disable build cache")
      ]),
      sub("ps", "List containers", [
        option("--all", "Include stopped containers"),
        option("--quiet", "Only IDs")
      ]),
      sub("images", "List images"),
      sub("pull", "Pull an image"),
      sub("push", "Push an image"),
      sub("exec", "Execute in a container", [
        option("-it", "Interactive terminal")
      ]),
      sub("logs", "Show container logs", [
        option("--follow", "Follow output"),
        option("--tail", "Limit lines", true)
      ]),
      sub("stop", "Stop a container"),
      sub("rm", "Remove a container"),
      sub("rmi", "Remove an image"),
      sub("compose", "Manage a Compose project"),
      sub("context", "Manage Docker contexts"),
      sub("system", "Manage Docker data")
    ],
  },
];
