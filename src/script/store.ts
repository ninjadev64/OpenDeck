const Store = require("electron-store");

const schema = {
	devices: {
		type: "object",
		default: {}
	},
	useBluetoothProntoKey: {
		type: "boolean",
		default: false
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

const store = new Store({ schema });
export default store;
