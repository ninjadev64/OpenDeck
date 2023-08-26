const { join } = require("path");
const { get } = require("https");
const { createWriteStream, unlink, readdirSync, readFileSync, rmSync } = require("fs");
const { getIcon } = require("../../build/shared");
const { ipcRenderer } = require("electron");

const AdmZip = require("adm-zip");
const path = require("path");
const store = require("../../build/store").default;
const dialog = require("dialog");

function download(url, dest, cb) {
	var file = createWriteStream(dest);
	get(url, (response) => {
		response.pipe(file);
		file.on("finish", function() {
			file.close(cb);
		});
	}).on("error", (err) => {
		unlink(dest, (err) => { if (err) throw err; });
		if (cb) cb(err.message);
	});
};

function install(plugin) {
	if (confirm(`Are you sure you want to install the plugin "${plugin.name}" by "${plugin.author.name}"?`)) {
		let path = join(store.get("userDataPath"), "Plugins", `${plugin.identifier}.streamDeckPlugin`);
		download(plugin.published_versions[0].direct_download_link, path, (err) => {
			if (err) throw err;
			new AdmZip(path).extractAllToAsync(join(store.get("userDataPath"), "Plugins"), true, false, (err) => {
				if (err) throw err;
				unlink(path, (err) => { if (err) throw err; });
				dialog.info(`Successfully downloaded and unpacked plugin "${plugin.name}"!`, "Success");
			});
		});
	}
}

let pluginDir = path.join(store.get("userDataPath"), "Plugins");
let pluginIDs = readdirSync(pluginDir, { withFileTypes: true }).filter((item) => item.isDirectory()).map((item) => item.name);
pluginIDs.forEach((p) => {
	let manifest = JSON.parse(readFileSync(path.join(pluginDir, p, "manifest.json"), "utf8"));
	let div = document.createElement("div");
		div.classList.add("plugin-card");

		let title = document.createElement("h3");
		title.innerText = manifest.Name;
		div.appendChild(title);

		let identifier = document.createElement("small");
		identifier.innerText = p.slice(0, -9);
		div.appendChild(identifier);

		let author = document.createElement("p");
		author.innerText = `by ${manifest.Author}`;
		div.appendChild(author);

		let button = document.createElement("button");
		button.innerText = "Remove";
		button.addEventListener("click", () => {
			Object.values(store.get("profiles")).forEach((profile) => {
				[].concat(profile.key, profile.slider).forEach((slot) => slot.forEach((instance) => {
					if (instance && instance.action.plugin == p) ipcRenderer.send("slotUpdate", instance.context, undefined);
				}));
			});
			div.remove();
			rmSync(path.join(pluginDir, p), { recursive: true, force: true });
		});
		div.appendChild(button);

		let icon = document.createElement("img");
		icon.src = getIcon(path.join(pluginDir, p, manifest.Icon));
		icon.classList.add("plugin-icon");
		div.appendChild(icon);

		document.body.appendChild(div);
});

document.body.appendChild(document.createElement("hr"));

fetch("https://appstore.elgato.com/streamDeckPlugin/catalog.flat.json").then(async (response) => {
	let data = await response.json();
	let entries = data.entries;
	for (const entry of entries) {
		if (entry.published_versions.length == 0 || entry.invisible) {
			continue;
		}

		let div = document.createElement("div");
		div.classList.add("plugin-card");

		let title = document.createElement("h3");
		title.innerText = entry.name;
		div.appendChild(title);

		let identifier = document.createElement("small");
		identifier.innerText = entry.identifier;
		div.appendChild(identifier);

		let author = document.createElement("p");
		author.innerText = `by ${entry.author.name}`;
		div.appendChild(author);

		let button = document.createElement("button");
		button.innerText = "Install";
		button.addEventListener("click", () => { install(entry); });
		div.appendChild(button);

		let icon = document.createElement("img");
		icon.src = entry.published_versions[0].icon_link;
		icon.classList.add("plugin-icon");
		div.appendChild(icon);

		document.body.appendChild(div);
	}
});