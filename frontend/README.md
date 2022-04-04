* Uses npm, avoid using yarn
* Use `.example.env.development` to set up `.env.development` for project
* Uses Ory Kratos for auth and account management
# Setup

* Requires Docker for running Ory Kratos

```
npm install
```


# Run in dev mode

Tab 1 - run Ory from docker
```
npm run ory:start
```

Tab 2 - run Svelte frontend
```
npm run dev
```