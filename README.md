> [!CAUTION]
> This is unstable and missing key security features, use is **strongly** discouraged!
# ratchet-pawl

This is a minimalist frontend for configuring Ratchet. It is draft-phase, and absolutely needs a bit more polish.

## Overview / Architecture
It uses [Rocket](https://rocket.rs) to produce some endpoints, which are enumerated in `fn rocket`

It uses [redb](https://redb.org) to provide persistence.

Also shout-out to [tabler's react icons](https://www.npmjs.com/package/@tabler/icons-react), very cool.

They allow definition of devices and users, as well as a backend API to be fetched by the ratchet TACACS daemon on launch.

## Future plans / Roadmap
- [ ] ❌ Data masking to prevent persisting keys in the clear.
- [ ] ❌ Memory hardening, like the Daemon to prevent keys getting persisted improperly.
- [ ] ❌ Bcrypt credential hashing to prevent persisting user passwords in the clear.
- [ ] ❓ assess pre-hashing architecture, so that
- [ ] Secure initialization / signup invite link
- [ ] Port from React to Dioxus
- [ ] Sand down the edges:
  - [ ] Ratchet -> ratchet
  - [ ] Toast notifications
  - [ ] Pop-over notification
  - [ ] Center justification
  - [ ] Stateful updates using Websockets
- [ ] Trouble monitoring
- [ ] Advanced security
  - [ ] Frontend lockdown / request filtering
  - [ ] This is common in this sort of application.
  - [ ] In the same vein, TLS certificate management for the webserver (or appropriate solution)
- [ ] Deployment architecture / Helm/Docker/K8s/etcd, Nullsoft, Rust-Crate, Snap, etc.
- [ ] Distributed replication architecture / fault tolerance / clustering

## Screenshots
It's not much to look at, very simple at this stage.
![image](https://github.com/user-attachments/assets/536b3a04-2b3c-4b2f-bd29-1f3d652fd89e)

## Building

You need to have npm
Suggestion: https://github.com/nvm-sh/nvm
- Just folow the instructions and install node it's pretty easy.

`git clone https://github.com/meltyness/ratchet-pawl`

`cd ratchet/pawl-js & npm run build &`

`cd ../ & cargo run --release`

