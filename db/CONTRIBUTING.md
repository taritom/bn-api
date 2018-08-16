# Contributing to Big Neon Database
[contributing-to-bn-db]: #contributing-to-bn-db

Thank you for your interest in Big Neon and Big Neon Database! We very much look forward to
your suggestions, bug reports, and pull requests.


If you have questions, please [join the conversation](#join-the-conversion). 

## Contributing code to Big Neon Database

In the ideal scenario, 

* you would [chat to other devs](#join-the-conversation), 
* have a browse through the [issue tracker] and/or the [project board] for something you think you can do, and put your hand up. 
* A maintainer would then assign the task to you, 
* and some time later you'll submit a [pull request](#pull-requests) with your contribution.
* Your PR passes all the automated checks (unit tests, code conventions, etc.) and 
* then it gets be merged into master by a maintainer.

Of course, you don't _have_ to follow this process to the letter, but this route offers the best and smoothest path for 
your code getting into the project, which is what we all want.  

### Setting up Big Neon locally

You'll want to have a local version of Big Neon for development and testing. This section explains how to set that up.

1. Install Rust using [rustup], which allows you to easily switch between Rust
   versions. Big Neon Database aims to support Rust stable.

2. Big Neon is a microservices-oriented architecture. There is a build script in the [big-neon repo](https://github.com/big-neon/bigneon)
   that helps you set up your local testing environment by pulling the various components as Docker images and setting
   them up a local network for you. See the [README](https://github.com/big-neon/bigneon/blob/master/README.md) for that 
   repo for more details.
   
3. To download and install _this project_, you can execute:

       git clone https://github.com/big-neon/bn-db
       cd bn-db
       cargo build

[rustup]: https://www.rustup.rs
[project board]: https://github.com/big-neon/bn-db/projects/

### Coding Style

We follow the [Rust Style Guide](https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/guide/guide.md), enforced 
using [rustfmt](https://github.com/rust-lang-nursery/rustfmt).
To run rustfmt tests locally:

1. Run `rustfmt` using cargo from the root of your Big Neon Database repo.
   
   To see changes that need to be made, run

   ```
   cargo fmt --all -- --write-mode=diff
   ```

   If all code is properly formatted (e.g. if you have not made any changes), this should run without error or output.
   If your code needs to be reformatted, you will see a diff between your code and properly formatted code.
   If you see code here that you didn't make any changes to then you are probably running the wrong version of rustfmt.
   Once you are ready to apply the formatting changes, run 

   ```
   cargo fmt --all
   ```

   You won't see any output, but all your files will be corrected.

You can also use rustfmt to make corrections or highlight issues in your editor.
Check out the [rustfmt README](https://github.com/rust-lang-nursery/rustfmt) for details.


## Pull Requests
[pull-requests]: #pull-requests

Pull requests are the primary mechanism we use to change Big Neon Database. GitHub itself
has some [great documentation][about-pull-requests] on using the Pull Request feature.
We use the "fork and pull" model [described here][development-models], where
contributors push changes to their personal fork and create pull requests to
bring those changes into the source repository.

[about-pull-requests]: https://help.github.com/articles/about-pull-requests/
[development-models]: https://help.github.com/articles/about-collaborative-development-models/

Please make pull requests against the `master` branch.

## Documentation

Documentation for the entire Big Neon project lives in the [Docs Repo](https://github.com/big-neon/docs).
Documentation specific to the Big Neon Database lives in the `docs` folder. Contributions to the documentation are welcome.  

## Bugs and Issues

Have a look at our Big Neon Database [issue tracker] for a list of outstanding issues.
Project-wide issues will also be listed on the [docs repo issue tracker](https://github.com/big-neon/docs/issues).

If you can't find an issue (open or closed) describing your problem (or a very similar one) there, please [open a 
new issue](https://github.com/big-neon/bn-db/issues/new) with the following details:

- What are you trying to accomplish?
- What is the full error you are seeing?
- How can we reproduce this?

[issue tracker]: https://github.com/big-neon/bn-db/issues
[Gist]: https://gist.github.com

## Submitting feature requests

If you can't find an issue (open or closed) describing your idea on our [issue tracker], open an issue. 
Adding answers to the following questions in your description wold be great:

- What do you want to do, and how do you expect Big Neon Database to support you with that?
- How might this be added to Big Neon Database?
- What are possible alternatives?
- Are there any disadvantages?

## Join the conversation

|   |   |
|---|---|
| <img src="https://ionicons.com/ionicons/svg/md-paper-plane.svg" height="32"/> | [Big Neon](https://t.me/bigneon) |
| <img src="https://ionicons.com/ionicons/svg/logo-twitter.svg" height="32"/>   | [@tari](https://twitter/tari) |
| <span style="font-size: 32px; font-weight: bold">#IRC</span>                  | #bigneon-dev |
