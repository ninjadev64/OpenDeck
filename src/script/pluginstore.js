const { join } = require("path");
const { get } = require("https");
const { createWriteStream, unlink } = require("fs");
const AdmZip = require("adm-zip");
const store = require("../script/store");
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

		let identifier = document.createElement("small");
		identifier.innerText = entry.identifier;

		let author = document.createElement("p");
		author.innerText = `by ${entry.author.name}`;

		let button = document.createElement("button");
		button.innerText = "Install";
		button.addEventListener("click", () => { install(entry); });

		let icon = document.createElement("img");
		icon.src = entry.published_versions[0].icon_link;
		icon.classList.add("plugin-icon");

		div.appendChild(title);
		div.appendChild(identifier);
		div.appendChild(author);
		div.appendChild(button);
		div.appendChild(icon);
		document.body.appendChild(div);
	}
});