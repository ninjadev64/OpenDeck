const path = require("path");
const { readdirSync, readFileSync } = require("fs");

const pluginsDir = path.join(localStorage.getItem("userData"), "Plugins");
const pluginDirs = readdirSync(pluginsDir, { withFileTypes: true })
    .filter((item) => item.isDirectory())
    .map((item) => item.name);
var plugins = [];

class StreamDeckPlugin {
    constructor(manifest) {
        this.name = manifest.Name;
        this.description = manifest.Description;
        this.author = manifest.Author;
        this.version = manifest.Version;
        this.website = manifest.URL;
        this.htmlPath = manifest.CodePath;
        this.iconPath = manifest.Icon;
        this.actions = [];
        
        manifest.Actions.forEach((action) => {
            this.actions.push(new Action(action.Name, action.UUID, action.Tooltip));
        });
    }
}


pluginDirs.forEach((dir) => {
    let manifest = readFileSync(path.join(pluginsDir, dir, "manifest.json"), "utf8");
    let pl = new StreamDeckPlugin(JSON.parse(manifest));
    plugins.push(pl);
});