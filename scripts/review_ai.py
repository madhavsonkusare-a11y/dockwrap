# One-off review text for the AI apps; applied per app once its proof passes.
import json, sys
from pathlib import Path
REVIEWS = {
    "anythingllm": (
        ["Asks on first open which model to use — a local one through Ollama, or a provider's API key, which it keeps in its data folder.",
         "Single-user and open by default: anyone who can reach the address can use it until you turn on password protection in its settings.",
         "Local Store does not grant the SYS_ADMIN capability upstream suggests for the browser AnythingLLM uses to read websites as documents; if adding a website fails, that is why."],
        "AnythingLLM 1.16.1, upstream's current release"),
    "big-agi": (
        ["Keeps nothing on the server: chats and API keys live in this app window's own storage on this computer, not in its folder. Deleting the app's data does not remove them, and they are tied to its address.",
         "Sends what you type to the model providers whose keys you enter."],
        "Big-AGI 2.1.1, upstream's current release"),
    "sillytavern": (
        ["Talks to whichever AI service you connect it to — a local model or a provider's API — with keys it keeps in its data folder.",
         "Open by default, with no login: anyone who can reach the address can use it. Its IP whitelist stays on, widened to Docker's own networks so the page reaches it through the port mapping.",
         "Characters and extensions you import come from other people; extensions are code that runs in the app."],
        "SillyTavern 1.18.0, upstream's current release"),
    "kotaemon": (
        ["Starts with one account, username admin and password admin; change it the first time you sign in.",
         "Uses the model you configure — a provider's API key or a local Ollama — and indexes the documents you upload into its folder.",
         "Kotaemon publishes 0.12.0 only as its rolling main-lite tag; the install pins the build from its release day, 2026-05-31, by digest.",
         "Its image is 15.9 GB — the largest Local Store offers — so the first install downloads that much and needs that much free disk."],
        "Kotaemon 0.12.0 (the release-day main-lite build)"),
    "langflow": (
        ["Opens without a login, in Langflow's automatic single-user mode: anyone who can reach the address can edit and run your flows.",
         "Flows can run Python code and call external services with keys you store in it; stored keys are encrypted with a key Local Store generated for this install."],
        "Langflow 1.12.1, upstream's current release"),
    "huginn": (
        ["The account is the username and password you chose at setup.",
         "Agents you create fetch web pages, send email and call web hooks on their schedules, from this computer.",
         "Keeps its database in a MariaDB 11.4 beside it, in its folder: MySQL 8 cannot keep its data on a Windows folder."],
        "Huginn v2026.09.09, upstream's current release"),
    "vane": (
        ["Asks on first open for a model provider and its key.",
         "Every question goes to the model provider you choose, and its searches go through the bundled SearXNG, which queries public search engines from this computer."],
        "Vane 1.12.2 (formerly Perplexica), upstream's current release"),
    "librechat": (
        ["Anyone who can reach the address can register an account; the first account you create is yours.",
         "Each person enters their own model API keys in the app; they are stored encrypted with a key Local Store generated.",
         "Its document-chat (RAG) service is not included, so uploading files for retrieval is unavailable."],
        "LibreChat 0.8.7, upstream's current release, with MongoDB 8.0.30 and Meilisearch 1.35.1 (the version LibreChat pins)"),
    "khoj": (
        ["Runs in single-user anonymous mode: it opens straight to the chat, and anyone who can reach the address can use it. The admin email and password you set open its /server/admin page.",
         "Sends questions to the model provider you configure and searches the web through a bundled SearXNG from this computer.",
         "Its code sandbox (Terrarium) is published only as :latest, so it is left out and Khoj cannot run code. Khoj 1.42.10 is what upstream's own :latest installs; 2.0 is still in beta."],
        "Khoj 1.42.10, the version upstream's own latest tag installs"),
    "flowise": (
        ["Asks you to create its administrator account the first time you open it.",
         "Flows call model providers and other services with credentials you store in it; they are encrypted with a key Local Store generated.",
         "Flowise is open core: features for its paid Enterprise edition stay locked."],
        "Flowise 3.1.4, upstream's current release"),
    "sim": (
        ["Asks you to create an account on first open; anyone who can reach the address can register one.",
         "Workflows call the model providers and services you add keys for; the keys are stored encrypted with a key Local Store generated."],
        "Sim 0.8.33, upstream's current release"),
    "maxun": (
        ["Asks you to create an account on first open.",
         "Robots load the websites you point them at from this computer, in a headless Chromium that runs with its own sandbox off, as upstream configures it; the container is what contains the pages it loads.",
         "Local Store runs that browser without the SYS_ADMIN capability and unconfined seccomp profile upstream's file asks for, which Chromium without its sandbox does not use.",
         "Screenshots are kept in a bundled RustFS object store, which replaces MinIO now that MinIO no longer publishes images. Telemetry to Maxun's developers is off."],
        "Maxun 0.0.62, upstream's current backend"),
    "lobehub": (
        ["Asks you to create an account on first open; anyone who can reach the address can register one.",
         "Uses the model providers you add keys for; the keys are stored encrypted with a key Local Store generated.",
         "Files you upload go to a bundled RustFS object store, a 1.0 release candidate, on a second loopback address the page uploads to directly.",
         "Its bucket is created by MinIO's mc client, whose last image is from September 2025; it talks only to the bundled store.",
         "Source-available under the LobeHub Community License, which restricts commercial derivative works."],
        "LobeHub 2.2.17, upstream's current release"),
    "dify": (
        ["Asks you to set up an administrator account the first time you open it.",
         "Code in your workflows runs in a sandbox that reaches only the internet, through a proxy that refuses this computer and its network; the agent sandbox is held the same way.",
         "Plugins install from Dify's marketplace into its folder and run inside its plugin service.",
         "Keeps vectors in its own PostgreSQL with pgvector rather than upstream's default Weaviate. Its Redis has no password, where upstream's has a publicly known default; only Dify's own containers can reach it.",
         "Source-available under the Dify Open Source License, Apache 2.0 with conditions on multi-tenant use and on removing its branding."],
        "Dify 1.17.1, upstream's current release"),
    "open-webui": (
        ["Asks you to create an account on first open; the first account is the administrator.",
         "Connects to an Ollama server at the address you give during setup, or to OpenAI with your key; other providers can be added in its settings.",
         "Source-available under the Open WebUI License, BSD-3 with a clause that keeps its branding."],
        "Open WebUI 0.11.3, upstream's current release"),
    "joplin": (
        ["Signs in first as admin@localhost with the password admin; change both in its settings, then connect your Joplin apps to its address."],
        "Joplin Server 3.7.1, with its database moved to PostgreSQL 14.24"),
}

# Written by Local Store, or taken from an app store's definition.
IMPORTED = {"open-webui": "Runtipi's app store", "joplin": "Runtipi's app store"}

def apply(app, extra_note=""):
    p = Path(f"src/templates/{app}.json"); d = json.loads(p.read_text(encoding="utf-8"))
    proof = json.loads(Path(d["lifecycle_proof"]).read_text(encoding="utf-8"))
    assert proof["passed"], f"{app} proof did not pass"
    notes, what = REVIEWS[app]
    d["risk_notes"] = [n for n in d["risk_notes"] if not n.startswith("REVIEW: ")] + notes
    creds = any("credentials it generated" in s["step"] for s in proof["steps"])
    import datetime
    today = datetime.date.today().isoformat()
    d["verified_at"] = today
    d["promotion"] = {"state": "approved", "reason": (
        f"Approved by the owner. Qualified as offered on {today}: a real install of {what} proved it starts, "
        f"opens to a first screen with a clear next step, survives a restart and a keep-data reinstall with its data"
        f"{' and generated credentials' if creds else ''} intact, and removes everything it created. "
        + (f"The definition comes from {IMPORTED[app]}." if app in IMPORTED
           else "Local Store wrote this definition from the project's own deployment instructions.")
        + extra_note)}
    p.write_text(json.dumps(d, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(app, "reviewed; markers left:", json.dumps(d).count("REVIEW: "))

if __name__ == "__main__":
    for app in sys.argv[1:]:
        apply(app)
