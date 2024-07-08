# StableKeepr

_AI Generated Art Manager_

[![Rust](https://github.com/fifth-interactive/StableKeepr/actions/workflows/rust.yml/badge.svg)](https://github.com/fifth-interactive/StableKeepr/actions/workflows/rust.yml)

Manage your AI-generated images with ease. Sort, filter, search, delete, and organize.

## Feature Roadmap

- [ ] Display images by folder or date
- [ ] View raw and parsed generation metadata
- [ ] Vector search image prompts. Prompt detection support for:
  - [ ] Comfyui
  - [ ] Automatic1111
  - [ ] Fooocus
  - _More to come_
- [ ] AI-generated image descriptions (also exposed in vector search)
- [ ] Integration with image generation tools:
  - [ ] ComfyUI
  - [ ] Internal Rust image generation
- [ ] Use of "Processes" to generate and modify images in steps without leaving StableKeepr
- [ ] Project view to see all images related to a named project
- [ ] Integration with CivitAI if possible (see [CivitAI](https://github.com/civitai/civitai))
  - Currently (according to their wiki), this integration may be very limited in scope. Eventually I would like to enable posting images directly from ImageKeepr. 

### Notice!

This project is a lot of "firsts" for me:
 * First time using Rust
 * First time using GTK (GUI framework)
 * First time writing a non-trivial desktop application
 * First time making a non-trivial public repository

In light of all these firsts, there will invariably be parts I mess up.

I am building StableKeepr because it is something that I want to use,
and my hope is that it will be useful to others as well.
I would love to be able to work on StableKeepr and things like it full time;
to that end, I plan on having StableKeepr available as AGPL for free,
and having "Professional" and "Enterprise" licenses available for a fee.
This may require dual licensing. I have not worked out all the details to that yet,
but I wanted to put that out there so if you want to contribute, you know what you are getting into.


