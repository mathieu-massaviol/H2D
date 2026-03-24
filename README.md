# H2D (hal2datacite)

An open-source desktop application for creating DataCite DOIs for [HAL](https://hal.science) (open archive) publications, and adding these DOIs back to HAL records.

## Features

- **Single DOI creation** — fetch metadata from HAL, map it to DataCite schema, validate and publish a DOI in a guided 3-step wizard
- **Batch DOI creation** — process multiple HAL records at once with progress tracking and CSV export of results
- **DOI list** — browse all DOIs in your DataCite account, filter by environment, and sync DOIs back to HAL records
- **Secure credential storage** — HAL and DataCite credentials are encrypted locally using [Stronghold](https://github.com/iotaledger/stronghold.rs)
- **Dual-environment support** — switch between production and test/preprod APIs without reconfiguring
- **Internationalization** — available in English and French

## Tech Stack

| Layer    | Technology                                          |
|----------|-----------------------------------------------------|
| Frontend | Vue 3, PrimeVue 4, Tailwind CSS 4, vue-i18n         |
| Backend  | Rust, Tauri v2                                      |
| Storage  | Tauri Stronghold plugin (encrypted vault)           |
| APIs     | HAL REST API, DataCite REST API                     |

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

## Getting Started

### Install dependencies

```bash
npm install
```

### Run in development mode

```bash
npm run tauri dev
```

This starts the Vite dev server and the Tauri window together with hot-module reload.

### Build for production

```bash
npm run tauri build
```

The compiled binary and installer will be placed in `src-tauri/target/release/bundle/`.

## Configuration

Before using the application, set up your credentials under the **Credentials** section:

| API       | Environment | Required fields               |
|-----------|-------------|-------------------------------|
| HAL       | Production  | Login, Password               |
| HAL       | Test        | Login, Password               |
| DataCite  | Production  | Repository ID, Password       |
| DataCite  | Test        | Repository ID, Password       |

Credentials are stored encrypted on your machine and never transmitted outside of API calls.

## Workflows

### Create a single DOI

1. Go to **Create DOI**
2. **Step 1 — Config**: choose environment (production/test), select your DOI prefix, and optionally set a custom suffix prefix
3. **Step 2 — Select document**: enter a HAL ID or HAL URL to fetch the record metadata
4. **Step 3 — Publish**: review the mapped DataCite metadata and publish the DOI; the DOI is then added back to the HAL record automatically

### Create DOIs in batch

1. Go to **Batch DOI**
2. Configure the environment, prefix, and optional suffix
3. Provide a list of HAL IDs
4. Review the documents in the overview step
5. Start processing — progress is tracked in real time
6. Download the results as a CSV file

### Browse and sync DOIs

1. Go to **My DOIs**
2. Browse all DOIs associated with your DataCite account
3. Use the sync action on any row to push a DOI back to the corresponding HAL record

## External APIs

| API      | Production                              | Test / Preprod                                |
|----------|-----------------------------------------|-----------------------------------------------|
| HAL      | https://api.archives-ouvertes.fr/       | https://api-preprod.archives-ouvertes.fr/     |
| DataCite | https://api.datacite.org/               | https://api.test.datacite.org/                |

## Project Structure

```
hal2datacite_project/
├── src/                    # Vue 3 frontend
│   ├── components/         # Reusable UI components
│   ├── views/              # Page-level views
│   ├── router/             # Vue Router configuration
│   ├── i18n/               # Translations (en, fr)
│   └── utils/              # Frontend utilities
├── src-tauri/              # Rust / Tauri backend
│   └── src/
│       ├── credentials.rs  # Secure credential management
│       ├── hal_service.rs  # HAL API integration
│       ├── datacite_service.rs  # DataCite API integration
│       └── lib.rs          # Tauri command registrations
├── public/                 # Static assets
└── index.html
```

## License

This project is open source. See [LICENSE](LICENSE) for details.
