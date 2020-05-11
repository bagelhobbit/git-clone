# Init files created by git

From [gitready.com](http://gitready.com/advanced/2009/03/23/whats-inside-your-git-directory.html)

Not all files are created during initialization

* HEAD

    The current ref that you're looking at, by default this will be `refs/heads/master`

* config

    Contains settings for this repository. This file is mostly used for defining where remotes live and some core settings, like if the repository is bare or not.

    Default config:

    ``` config
    [core]
        bare = false
        repositoryformatversion = 0
        filemode = false
        symlinks = false
        ignorecase = true
        logallrefupdates = true
    ```

* description

    If you're using `gitweb` of firing up `git instaweb`, this will show up when you view your repository or the list of all versioned repositories

    Defaults to

    ``` config
    Unnamed repository; edit this file 'description' to name the repository.
    ```

* hooks

    This contains scripts that are executed at certain times when working with git, such as after a commit or before a rebase

    These files are not created directly, but git creates `*.sample` files for each

  * applypatch-msg
  * commit-msg
  * post-commit
  * post-receive
  * post-update
  * pre-applypatch
  * pre-commit
  * pre-rebase
  * prepare-commit-msg
  * update

* index

    The staging area with metadata such as timestamps, file names, and SHAs of the files that are already wrapped up by Git

* info
  * exclude

    File that can be used to ignore files for this project. It is *not* versioned like a `.gitignore` file would be

* logs

    Contains history for different branches. Seems to be used mostly with the `reflog` command

  * HEAD
  * refs

* objects

    Git's internal warehouse of blobs, all indexed by SHAs

* refs

    The master copy of all refs that live in this repository, be they for stashes, tags, remote tracking branches, or local branches

  * heads
  * remotes
  * stash
  * tags
  
* branches

    unknown

* COMMIT_EDITMSG

    The last commits message

* FETCH_HEAD

    The SHAs of branch/remote heads that were updated during the last `git fetch`

* ORIG_HEAD

    When doing a merge, this is the SHA of the branch you're merging into

* MERGE_HEAD

    When doing a merge, this is the SHA of the branch you’re merging from

* MERGE_MODE

    Used to communicate constraints that were originally given to git merge to git commit when a merge conflicts, and a separate git commit is needed to conclude it. Currently --no-ff is the only constraints passed this way

* MERGE_MSG

    Enumerates conflicts that happen during your current merge
