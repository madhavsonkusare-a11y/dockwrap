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
         "Open by default, with no login: anyone who can reach the address can use it. Its IP whitelist is off because the port is reachable only from this computer.",
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
         "Runs its own MySQL inside the same container, keeping the database in its folder."],
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
}

def apply(app, extra_note=""):
    p = Path(f"src/templates/{app}.json"); d = json.loads(p.read_text(encoding="utf-8"))
    proof = json.loads(Path(d["lifecycle_proof"]).read_text(encoding="utf-8"))
    assert proof["passed"], f"{app} proof did not pass"
    notes, what = REVIEWS[app]
    d["risk_notes"] = [n for n in d["risk_notes"] if not n.startswith("REVIEW: ")] + notes
    creds = any("credentials it generated" in s["step"] for s in proof["steps"])
    d["verified_at"] = "2026-09-11"
    d["promotion"] = {"state": "approved", "reason": (
        f"Approved by the owner. Qualified as offered on 2026-09-11: a real install of {what} proved it starts, "
        f"opens to a first screen with a clear next step, survives a restart and a keep-data reinstall with its data"
        f"{' and generated credentials' if creds else ''} intact, and removes everything it created. "
        f"Local Store wrote this definition from the project's own deployment instructions.{extra_note}")}
    p.write_text(json.dumps(d, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(app, "reviewed; markers left:", json.dumps(d).count("REVIEW: "))

if __name__ == "__main__":
    for app in sys.argv[1:]:
        apply(app)
