# BlockVisor

The service that runs on the host systems and is responisble for provisioning and managing one or more blockchains on a single server.

## How to release a new version

1. Update version in workspace
2. Run `make tag` to create a version tag
3. CI `publish` workflow will then build a bundle and create a new GH release

## API proto files

API proto files are stored in [separate repository](https://github.com/blockjoy/api-proto).

Note that [git submodules](https://github.blog/2016-02-01-working-with-submodules/) are used to bring the protos to this project.

```
git submodule update --init --recursive
```

# High Level Overview

![](overview.jpg)

# Basic Scenarios
## Add Host - Host Provisioning

```mermaid
sequenceDiagram
    participant user as User
    participant frontend as Fronted
    participant backend as API
    participant host as Host
    participant bv as BlockvisorD
    
    user->>frontend: add new Host
    frontend->>backend: new Host
    backend-->>frontend: OTP
    frontend-->>user: provisioning script with OTP
    user->>host: run provisioning script with OTP
    host->>host: download images and start BlockvisorD with OTP
    bv->>backend: ProvisionHostRequest
    backend-->>bv: ProvisionHostResponse
```

## Add Node

### Overview

```mermaid
sequenceDiagram
    participant backend as API
    participant bv as BlockvisorD
    participant fc as Firecracker
    participant babel as Babel
    
    backend->>bv: NodeCreate
    bv-->>backend: InfoUpdate
    bv->>fc: create vm
    backend->>bv: NodeStart
    bv-->>backend: InfoUpdate
    bv->>fc: start vm with Babel inside
    babel->>fc: listen for messages on vsock
```

### More detailed view including key exchange and node initialization

```mermaid
sequenceDiagram
    participant frontend as Frontend
    participant backend as API
    participant bv as BV
    participant babel as Babel

    frontend ->> backend: Create Node
    backend ->> bv: Create Node
    bv ->> bv: Download os.img
    bv ->> bv: Create data.img

    frontend ->> backend: Start Node
    backend ->> bv: Start Node

    bv ->> babel: Start Node
    babel ->> babel: Load config
    babel ->> babel: Mount data.img
    babel ->> babel: Supervisor: wait for init

    bv ->> babel: Ping
    babel -->> bv: Pong
    bv ->> babel: Setup genesis block
    babel ->> babel: Init completed
    babel ->> babel: Start Supervisor
    babel ->> babel: Start blockchain

    bv ->> backend: Get keys
    backend -->> bv: Keys?
    alt Got keys
        bv ->> babel: Setup keys
    else No keys found
        bv ->> babel: Generate new keys
        babel -->> bv: Keys
        bv ->> backend: Save keys
    end

    opt Restart needed
        babel ->> babel: Supervisor: restart processes
    end

    frontend ->> backend: Get keys
    backend -->> frontend: Keys
```

## Execute Method on Blockchain

```mermaid
sequenceDiagram
    participant cli as BV CLI
    participant bv as BlockvisorD
    participant babel as Babel

    cli->>bv: Blockchain Method
    bv->>babel: send(method)
    babel->>babel: map method to Blockchain API as defined in config.toml
    babel-->>bv: response
    bv-->>cli: response
```

## Self update processes

![](host_self_update.jpg)

### Check for update

```mermaid
sequenceDiagram
    participant repo as BlockJoy Repo
    participant bv as BlockvisorD
    participant installer as BlockvisorD.installer
    
    bv ->> repo: check for update on startup
    activate repo
    repo -->> bv: no updates
    deactivate repo
    bv ->> repo: subscribe update notifications
    repo ->> bv: new release notification
    bv ->> repo: download latest version
    activate repo
    repo -->> bv: latest bundle with installer
    deactivate repo
    bv ->> installer: launch installer
```

### BlockvisorD update

```mermaid
sequenceDiagram
    participant api as API
    participant bv as BlockvisorD
    participant installer as BlockvisorD.installer
    participant sysd as SystemD
    
    alt running version is not blacklisted (this is not rollback)
        installer ->> installer: backup running version
    end
    alt BlockvisorD is running
        installer ->> bv: notify about started update
        activate bv
        bv ->> api: change status to UPDATING
        bv ->> bv: finish pending actions    
        deactivate bv
    end
    installer ->> installer: switch current version (change symlinks)  
    installer ->> sysd: restart BlockvisorD service     
    sysd ->> bv: restart BlockvisorD service
    installer ->> bv: finalize update and get health check status
    activate bv
    alt update succeed
        bv ->> bv: update Babel on running nodes
        bv ->> api: change status to IDLE
        bv -->> installer: ok
        installer ->> installer: cleanup old version
    else update failed
        bv -->> installer: err
        installer ->> installer: mark this version as blacklisted
        alt was it rollback
            installer ->> api: send rollback error (broken host) notification
        else
            installer ->> installer: launch installer from backup version
        end
    end
    deactivate bv
```

### Babel install

```mermaid
sequenceDiagram
    participant bv as BlockvisorD
    participant fc as Firecracker
    participant babelvisor as BabelvisorD
    participant babel as Babel
    
    bv ->> fc: start node  
    activate babelvisor
    bv ->> babelvisor: send Babel binary
    babelvisor ->> babel: start
    alt failed to start Babel
        babelvisor ->> bv: Babel start error
    end
    deactivate babelvisor
```

### Babel update

```mermaid
sequenceDiagram
    participant bv as BlockvisorD
    participant babelvisor as BabelvisorD
    participant babel as Babel
   
    bv ->> babelvisor: get Babel version
    activate babelvisor
    babelvisor -->> bv: Babel version
    deactivate babelvisor
    alt version doesn't match expected
        bv ->> babelvisor: send Babel binary
        activate babelvisor
        babelvisor ->> babelvisor: replace Babel binary
        activate babel
        babelvisor -->> babel: request graceful shutdowan
        babel ->> babel: finish pending actions
        babel ->> babel: graceful shutdown
        deactivate babel
        babelvisor ->> babel: restart
        activate babel
        babelvisor ->> babel: health check
        alt uodated version health check pass
            babelvisor ->> bv: Babel update done
        else
            deactivate babel 
            babelvisor ->> bv: Babel update error
        end
        deactivate babelvisor
    end
```
