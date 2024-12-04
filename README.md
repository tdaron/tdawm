<a id="readme-top"></a>



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->



<!-- PROJECT LOGO -->
<br />
<!-- <div align="center"> -->
  <!-- <a href="https://github.com/github_username/repo_name"> -->
    <!-- <img src="images/logo.png" alt="Logo" width="80" height="80"> -->
  <!-- </a> -->

<h3 align="center">TDAWm</h3>
  <strong>This projet has reached a stopping point as I move to Wayland for hidpi screens support :( </strong>

  <p align="center">
    This project is a <bold>X11</bold> Window Manager written in Rust.
    It's not meant to be the fastest, the lightest, the more extensible or whatever.
    I just wanted something simple that just works; I used to work with dwm but I wanted something
    I could better fit to my needs.
    <br />
    This project has simplicity in mind. I want the code to be clear and the end product a breath to use.
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#built-with">Built With</a>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>





### Built With

This project is made in Rust.
<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started


### Prerequisites

The only prerequisite is X11. You probably already have it otherwise you wouldn't be here i guess.

### Installation

There is no package distribution so at the moment you will have to clone this repository

    git clone https://github.com/tdaron/tdawm

and then build it using cargo

    cargo build --release

The executable will be placed in `target/release/tdawm`.

Simply put it inside your .xinitrc and there you go !

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

There is still not any documentation as this project is still a POC atm.
<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [x] Basic X11 window manager (handle events)
- [x] Basic layouts (horizontal + vertical)
- [x] Basic workspaces
- [ ] Multiple screens support (using Xinemara) 
- [ ] Basic (EWMH)[https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints] protocol support
- [ ] 2D Workspaces (experiment)
- [ ] Linked workspaces between screens (mirroring some workspaces between 2 screens) 

See the [open issues](https://github.com/tdaron/tdawm/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Théo Daron - theo@daron.be

Project Link: [https://github.com/tdaron/tdawm](https://github.com/tdaron/tdawm)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments


<p align="right">(<a href="#readme-top">back to top</a>)</p>
