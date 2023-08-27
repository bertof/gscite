# Gscite

![Crates.io](https://img.shields.io/crates/l/gscite?style=flat-square)
![GitHub Stars](https://img.shields.io/github/stars/bertof/gscite?style=flat-square)
![GitHub License](https://img.shields.io/github/license/bertof/gscite)

Execute
Scraper for Google Scholar written in Rust

This library has first been built from scratch to automatically update BibTex metadata.
Later, some more advanced query functionalities have been added following [gscholar](https://lib.rs/crates/gscholar). Implementation. Unfortunately, Gscholar doesn't expose the underlying features of its dependency `reqwest`, so I preferred to extend my own library.
