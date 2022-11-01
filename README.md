# RSS Blue Tools

A collection of tools for public use created at [RSS Blue](https://rssblue.com/).

## Code

The app is built using WebAssembly.
Although it could be served statically, a lightweight server is used to render the `<head>` (`meta` and `title` tags, which are different for each page).

- `src` contains server-side code
- `app` contains client-side code
