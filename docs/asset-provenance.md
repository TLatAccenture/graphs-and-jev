# Asset provenance

These marks identify products in technical documentation. They are stored locally so the publication does not hotlink assets. Product names and marks belong to their owners. Their inclusion does not imply endorsement, sponsorship or partnership.

## Google Cloud architecture marks

The six new files were retrieved from the [Google Cloud official icon library](https://cloud.google.com/icons) core icon pack on 13 October 2026. Their contents are byte-identical to the SVGs in the downloaded pack. Only the local filenames changed.

| Local file | Product or category | Official source and original file | Retrieval date | Embedded metadata retained | Transformation |
| --- | --- | --- | --- | --- | --- |
| `static/assets/gcp-icons/compute-engine.svg` | Google Cloud Compute Engine | [Google Cloud icon library](https://cloud.google.com/icons), `Unique Icons/Compute Engine/SVG/ComputeEngine-512-color-rgb.svg` | 13 October 2026 | XML declaration, SVG identifiers and Adobe Illustrator generator comment retained; no embedded title or licence element | None; byte-identical content, local filename changed |
| `static/assets/gcp-icons/alloydb.svg` | Google Cloud AlloyDB | [Google Cloud icon library](https://cloud.google.com/icons), `Unique Icons/AlloyDB/SVG/AlloyDB-512-color.svg` | 13 October 2026 | XML declaration, SVG identifiers and Adobe Illustrator generator comment retained; no embedded title or licence element | None; byte-identical content, local filename changed |
| `static/assets/gcp-icons/cloud-run.svg` | Google Cloud Run | [Google Cloud icon library](https://cloud.google.com/icons), `Unique Icons/Cloud Run/SVG/CloudRun-512-color-rgb.svg` | 13 October 2026 | XML declaration, SVG identifiers and Adobe Illustrator generator comment retained; no embedded title or licence element | None; byte-identical content, local filename changed |
| `static/assets/gcp-icons/cloud-storage.svg` | Google Cloud Storage | [Google Cloud icon library](https://cloud.google.com/icons), `Unique Icons/Cloud Storage/SVG/Cloud_Storage-512-color.svg` | 13 October 2026 | XML declaration, SVG identifiers and Adobe Illustrator generator comment retained; no embedded title or licence element | None; byte-identical content, local filename changed |
| `static/assets/gcp-icons/bigquery.svg` | Google Cloud BigQuery | [Google Cloud icon library](https://cloud.google.com/icons), `Unique Icons/BigQuery/SVG/BigQuery-512-color.svg` | 13 October 2026 | XML declaration, SVG identifiers and Adobe Illustrator generator comment retained; no embedded title or licence element | None; byte-identical content, local filename changed |
| `static/assets/gcp-icons/looker.svg` | Google Cloud Looker | [Google Cloud icon library](https://cloud.google.com/icons), `Unique Icons/Looker/SVG/Looker-512-color.svg` | 13 October 2026 | XML declaration, SVG identifiers and Adobe Illustrator generator comment retained; no embedded title or licence element | None; byte-identical content, local filename changed |

### ADK and Gemini marks (owner decision, 13 October 2026)

None of the Google Cloud icon packs has a unique mark for ADK or Gemini. The owner chose the official ADK logo from the ADK documentation and the Gemini product logo used on the [Gemini Enterprise Agent Platform documentation](https://docs.cloud.google.com/gemini-enterprise-agent-platform). That page loads `https://www.gstatic.com/images/branding/productlogos/gemini_2025/v1/192px.svg` (about 3 MB, with embedded raster images); the local copy is Google's published 96 px PNG of the same logo from the same branding path.

| Local file | Product or category | Official source and original file | Retrieval date | Embedded metadata retained | Transformation |
| --- | --- | --- | --- | --- | --- |
| `static/assets/gcp-icons/adk.png` | Agent Development Kit (ADK) | [ADK documentation](https://google.github.io/adk-docs/), `https://google.github.io/adk-docs/assets/agent-development-kit.png` (512 x 512 PNG) | 13 October 2026 | PNG as published | None; byte-identical, local filename changed |
| `static/assets/gcp-icons/gemini.png` | Gemini | Google product logo, `https://www.gstatic.com/images/branding/productlogos/gemini_2025/v1/web-48dp/logo_gemini_2025_color_2x_web_48dp.png` (96 x 96 PNG), as used on the [Gemini Enterprise Agent Platform documentation](https://docs.cloud.google.com/gemini-enterprise-agent-platform) | 13 October 2026 | PNG as published | None; byte-identical, local filename changed |

## Existing engine marks

These records were ported from the engine-logo provenance README in the source application. All were retrieved on 2 October 2026.

| Local file | Product or category | Official source and original file | Retrieval date | Embedded metadata retained | Transformation |
| --- | --- | --- | --- | --- | --- |
| `static/assets/engine-logos/neo4j.svg` | Neo4j | [Neo4j full-colour logo](https://dist.neo4j.com/wp-content/uploads/20230926084108/Logo_FullColor_RGB_TransBG.svg) and [brand portal](https://neo4j.com/brand/) | 2 October 2026 | Source SVG metadata retained | Trailing whitespace trimmed; paths, colours and proportions unchanged |
| `static/assets/engine-logos/falkordb.png` | FalkorDB | [Official FalkorDB repository](https://github.com/FalkorDB/FalkorDB), published README image `FalkorDB Logo Square B` | 2 October 2026 | Original PNG metadata retained | None; original 508 by 508 PNG used unchanged |
| `static/assets/engine-logos/memgraph.svg` | Memgraph | [Memgraph documentation navigation logo](https://memgraph.com/docs/memgraph-logo-navigation.svg) | 2 October 2026 | Source SVG metadata retained apart from generator comment | Adobe Illustrator generator comment removed and trailing whitespace trimmed; paths, colours and proportions unchanged |
| `static/assets/engine-logos/google-cloud-spanner.svg` | Google Cloud Spanner, used for Spanner Graph | [Google Cloud icon library](https://cloud.google.com/icons), core pack `Unique Icons/Cloud Spanner/SVG/CloudSpanner-512-color.svg` | 2 October 2026 | XML declaration and SVG identifiers retained; no embedded title or licence element | Adobe Illustrator generator comment removed; paths, colours and proportions unchanged |
| `static/assets/engine-logos/puppygraph.svg` | PuppyGraph | [PuppyGraph horizontal purple logo](https://cdn.prod.website-files.com/65d609edcc331dd0e4eb5167/65d7517f9f226df9af7bf547_Horizontal_purple.svg) | 2 October 2026 | Source SVG metadata retained | None recorded |
| `static/assets/engine-logos/microsoft-fabric.svg` | Microsoft Fabric | [Microsoft Fabric icons](https://learn.microsoft.com/en-us/fabric/fundamentals/icons), package `@fabric-msft/svg-icons` under the MIT licence, original `fabric_48_color.svg` | 2 October 2026 | Package licence metadata retained in the source record | Added `viewBox="0 0 48 48"` matching the source canvas; paths, colours and proportions unchanged |
| `static/assets/engine-logos/mongodb.svg` | MongoDB Atlas | [MongoDB brand resources](https://www.mongodb.com/company/newsroom/brand-resources), original horizontal default slate-blue wordmark | 2 October 2026 | Source SVG metadata retained | None recorded |
| `static/assets/engine-logos/azure-cosmos-db.svg` | Azure Cosmos DB for Apache Gremlin | [Azure architecture icons](https://learn.microsoft.com/en-us/azure/architecture/icons/), Azure Public Service Icons V24, original `10121-icon-service-Azure-Cosmos-DB.svg` | 2 October 2026 | Source SVG metadata retained | None recorded |

Every mark appears beside its product name and keeps its original aspect ratio. Image alternative text is empty where adjacent visible text already carries the meaning. The SVG regression gate rejects script-bearing or active elements, event-handler attributes, CSS imports and executable or remote references.
