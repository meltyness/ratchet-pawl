> [!WARNING]
> This is unstable and missing some security features, use is discouraged!
# ratchet-pawl

This is a minimalist frontend for configuring Ratchet. It is draft-phase, and absolutely needs a bit more polish.

See https://github.com/meltyness/ratchet-cycle for a neat way to deploy the whole thing.

## Overview / Architecture
It uses [Rocket](https://rocket.rs) to produce some endpoints, which are enumerated in `fn rocket`

It uses [redb](https://redb.org) to provide persistence.

Also shout-out to [tabler's react icons](https://www.npmjs.com/package/@tabler/icons-react), very cool.

They allow definition of devices and users, as well as a backend API to be fetched by the ratchet TACACS daemon on launch.

`pawl` forms a sort of publisher over `ratchet` instances, but currently it can only realistically tolerate a single `ratchet` correctly.

## Future plans / Roadmap
- [ ] ❌ Detailed input sanitization / validation
- [ ] ❓ assess pre-hashing architecture, so that
- [ ] Secure initialization / signup invite link
- [ ] Port from React to Dioxus
- [ ] Sand down the edges:
  - [x] I don't like how the logout button is aligned
  - [ ] Port 80 redirection 
  - [ ] Ratchet -> ratchet
  - [ ] Toast notifications
  - [ ] Pop-over notification
  - [ ] Center justification
  - [ ] Stateful updates using Websockets
  - [ ] key entry, not *password* entry; password managers shouldn't offer
- [ ] Trouble monitoring
- [ ] Advanced security
  - [ ] Frontend lockdown / request filtering
  - [ ] This is common in this sort of application.
  - [ ] In the same vein, TLS certificate management for the webserver (or appropriate solution)
- [ ] Distributed replication architecture / fault tolerance / clustering
- [ ] wholistically address 'pub-sub' between `ratchet` and `pawl`

## Built

- [x] safer cookie disposal
- [x] Deployment architecture / Helm/Docker/K8s/etcd, Nullsoft, Rust-Crate, Snap, etc.
- [x] ❌ Data masking to prevent persisting keys in the clear.
- [x] ❌ Bcrypt credential hashing to prevent persisting user passwords in the clear.
- [x] ❌ Memory hardening, like the Daemon to prevent keys getting persisted improperly.

## Screenshots
It's not much to look at, very simple at this stage.
![image](https://github.com/user-attachments/assets/536b3a04-2b3c-4b2f-bd29-1f3d652fd89e)

## Building
I suspect it only runs on Linux
You need to have npm and cargo installed

just do:

```bash
cargo install --git https://github.com/meltyness/ratchet-pawl
RATCHET_PAWL_MASKING_KEY="must_specify_a_key" ratchet-pawl
```

Your shell will display some credentials to try it out.
