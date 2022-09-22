<p align="center">
<img src="https://user-images.githubusercontent.com/66782780/191822589-31634263-599a-41b8-b16d-439d7065fc3b.svg" alt="Lekh text editor"><br>
  
<img src="https://img.shields.io/badge/Made%20with-Rust-%23ff3300" alt="Made with Rust">
<img src="https://img.shields.io/badge/Supports-Syntax%20highlighting-%23cc0099" alt="Supports Syntax highlighting">
<img src="https://img.shields.io/badge/Compatibility-Cross--platform-%239966ff" alt="Compatibility Cross-platform">
<br>
  
<h1 align="center"> Lekh Text Editor </h1>
</p>

<p align="center">
A simple cross-platform text editor written in Rust programming language that supports syntax highlighting for a large number of programming languages, incremental search and window resize handling. This project is made as a learning experience and does not aim to be the best / the fastest text editor out there.
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#key-features">Key Features</a> •
  <a href="#how-to-use">How To Use</a>
</p>

<p align="center"><img src="https://user-images.githubusercontent.com/66782780/191851394-24944d86-3cde-4ca9-bbca-c17114cbff40.png" width="700"></p>  

## Installation
  * **Build from source**
    ```bash
    git clone https://github.com/appleswiggy/lekh.git
    cd lekh/
    cargo build --release
    ./target/release/lekh [FILENAME]
    ```
## Key Features
  * **Syntax Highlighting**  
    * Lekh supports dynamic syntax highlighting where each row is aware of the context of full document. If a change happens in one row of the document, the other rows re-highlight themselves to match the context otherwise the change could potentially render the highlighting invalid.  
    * Lekh automatically detects and loads the appropriate syntax for the file using its file name or by reading the first line of the file.  
<p align="center"><img src="https://user-images.githubusercontent.com/66782780/191846117-5b509f48-bc3d-4759-b03f-1c7a90b9195e.png" width="700"></p>

  * **Incremental Search**
    * Lekh supports incremental search where the file is searched after each key press when the user is typing in their search query.
    * User can search forward or backward using the arrow keys.
<p align="center"><img src="https://user-images.githubusercontent.com/66782780/191851648-2ba2e871-5a23-49e8-bfae-eab3f35d09c6.png" width="700"></p>  

## How To Use
* To open an empty text editor window, execute the binary:
  ```bash
  lekh
  ```
* To open a file from its path:
  ```bash
  lekh [FILEPATH]
  ```
