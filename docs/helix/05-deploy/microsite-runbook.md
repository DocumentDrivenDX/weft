---
ddx:
  id: weft.microsite-runbook
  type: runbook
  activity: deploy
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.microsite-design
      kind: informed_by
---

# Signed microsite deployment

GitHub Pages serves https://documentdrivendx.github.io/weft/ through .github/workflows/pages.yml. A static Node build copies website files and the brand/design documents, then pinned Innsigle signs and verifies every content file and its manifest. Claims and the issuer public key are published at .well-known/innsigle/. Signing failures stop artifact upload and deployment. The manifest inventories exact bytes; verification metadata is not recursively signed.

The dedicated Weft build key is held only in the repository Actions secret INNSIGLE_BUILD_KEY. The public key is committed under .innsigle/public/keys.json. The issuer is a project build identity, not a human endorsement. No private key, credentials or provenance transcript goes into the deploy artifact. Model-primary composition is explicit; no invented human-input percentage is reported.

Local build: node scripts/build-site.mjs. Signing: supply INNSIGLE_CLI pointing to the pinned Innsigle CLI and INNSIGLE_KEY_FILE pointing to an authorized private key, then node scripts/sign-site.mjs. CI performs the same byte verification. For a manual deployment, dispatch the Deploy signed Weft microsite workflow from main. To roll back, revert the website change through the repository workflow and deploy the reverted source; CI generates fresh seals for those bytes. Never publish stale claims or bypass signing.

Check desktop/mobile layout, keyboard tabs, section navigation, absence of page errors and horizontal viewport overflow before release. There is no analytics or RUM collection; Web Vitals should be measured in a real browser without asserting invented field metrics. If a page signature fails, do not treat the content as verified: rebuild and inspect the changed bytes. An action failure leaves the previous Pages deployment available.
