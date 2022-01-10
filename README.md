# Overview
graphene is an abstraction library over the various graphic APIs with a specific focus on 2D and application development.
graphene is part of a larger application development framework called [alloy](https://github.com/Folling/alloy).

This document is a WIP and will be expanded upon as the project progresses.

# Supported APIs
- [ ] OpenGL
- [ ] Vulkan
- [ ] DirectX
- [ ] Metal

This library has existed in multiple forms before. Two C++ versions and one previous rust version. 
You can find these projects (in ascending order of the creation date):
- [Original C++ version](https://memleak.eu/Folling/graphite)
- [Original Rust version](https://memleak.eu/Folling/graphite-rs)
- [Second C++ iteration](https://memleak.eu/Folling/graphite-CPP-v2)

# Features
- [ ] Shaders & Shaderprograms
- [ ] Textures
- [ ] Vertex Pipelines

# Compatiblity
The minimum OpenGL version used is 3.3. This is because of the following required features:
- Dual Source Blending
- Multisampled Framebuffers

Other APIs haven't been looked at in detail yet and won't be necessary for alloy in the near future. 
Should any restrictions come up during their installation they will be documented here.

# Contributing
alloy is open source and is supposed to benefit from the inclusion of other people. 
However I do reserve the rights to deny any feature requests or pull requests but am always open to discussion and having my mind changed. 
If you're uncertain whether or not a certain pull request would be appreciated and don't want to waste effort without knowing whether it's worth it, feel free to open an issue and ask. 
All code should be formatted using the same guideline. For this please use rustfmt. In the future a customised rustfmt stylisation might be used.
File and directory names are are to be formatted using snake_case. Excluded from this rule are files that have a certain convention such as .gitignore, LICENCE.txt and markdown files.

# Support
I have a fulltime job and can only afford so much time for alloy (the root project of graphite). If you would like to change that in the future consider donating to the project (note: Donating link will follow, graphite isn't worth donating yet). I also appreciate feedback (next to constructive criticism) so feel free to email me at coding@folling.de. 
