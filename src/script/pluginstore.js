const { join } = require("path");
const { get } = require("https");
const { createWriteStream, unlink } = require("fs");
const AdmZip = require("adm-zip");
const store = require("../script/store");

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

fetch("https://appstore.elgato.com/streamDeckPlugin/catalog.flat.json").then(async (response) => {
	let data = await response.json();
	let entries = data.entries;
	document.getElementById("install").addEventListener("click", () => {
		let plugin = entries.find(item => item.identifier == document.getElementById("identifier").value);
		if (confirm(`Are you sure you want to install the plugin "${plugin.name}" by "${plugin.author.name}"?`)) {
			let path = join(store.get("userDataPath"), "Plugins", `${plugin.identifier}.streamDeckPlugin`);
			download(plugin.published_versions[0].direct_download_link, path, (err) => {
				if (err) throw err;
				new AdmZip(path).extractAllToAsync(join(store.get("userDataPath"), "Plugins"), true, false, (err) => {
					if (err) throw err;
					unlink(path, (err) => { if (err) throw err; });
					alert(`Successfully downloaded and unpacked plugin "${plugin.name}"!`);
				});
			});
		}
	});
});