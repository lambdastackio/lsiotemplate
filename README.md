## Handlebars Template CLI

Install via Cargo:
>cargo install lsiotemplate

OR

>Clone and build as a normal Rust binary

Homebrew and other packages will come later

Uses handlebars template language to allow you add template variables to files and then dynamically replace them via the command line or via an input *JSON* or *YAML*  or *INI* file and then output the results to a file or stdout.

Very useful for build environments that need to change based on data.

Uses handlebars since it's the most widely used for Rust code and works well in a server and client environment (using javascript).
