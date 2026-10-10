# LEFT JOIN compiler qualification

This evidence qualifies the explicit public v0.3 LEFT JOIN compiler subset and its native CLI, Python and real Chromium compile transports. It does not qualify native database LEFT JOIN execution, Ashlar publication readability, or production authority.

`original-qualification/` preserves the exact reviewed 23-source freeze from a71cf7cbb9de5387d3763cfd781419440a11adf2: seven request/response schema pairs and transport cases, 52 unchanged historical artifact replays, source/ABI/boundary checks, and the original fingerprint manifest. Browser startup and double-serialization failures remain separate from the corrected successful run.

`current-base-integration/` preserves the new CLI and integration on ee90571a5aa67b6d2e6069220f6e1cc0eec3822c. All seven raw browser-input cases reproduce the original outputs; all 52 historical artifacts remain unchanged. The integration retains the bounded CLI, verifies four bounded-CLI tests, 53 compiler tests, 32 core tests, 18 backend-pipeline tests, workspace ABI compilation, three real boundary controls and an actual forbidden feature collision. Its source custody records the project Cargo/Rust inputs. Reproduction requires the complete ee90571a5aa67b6d2e6069220f6e1cc0eec3822c checkout with the 23 reviewed changes and its unchanged locked/vendor dependencies.

Both original executable realizations are retained unchanged. Their different binary digests identify different compiled source bases; matching case outputs do not make them interchangeable. The outer fingerprint manifest binds both unchanged inner manifests and every retained file.
