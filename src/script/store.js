let randomDefaultProfileId = Date.now().toString(36) + Math.random().toString(36).substring(2);
const schema = {
	devices: {
		type: "object",
		default: {}
	},
	webSocketPort: {
		type: "number",
        minimum: 0,
        maximum: 65535,
		default: 57116
	},
	propertyInspectorPort: {
		type: "number",
        minimum: 0,
        maximum: 65535,
		default: 57117
	},
	userDataPath: {
		type: "string"
	},
	autoLaunch: {
		type: "boolean",
		default: true
	},
	pluginSettings: {
		type: "object",
		default: {}
	},
	bundleIDs: {
		type: "object",
		default: {}
	}
}

const Store = require("electron-store");
const store = new Store({ schema });

module.exports = store;