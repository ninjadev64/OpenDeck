const schema = {
	keys: {
		type: "array",
        default: [
            undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined
        ]
	},
	sliders: {
		type: "array",
		default: [ undefined, undefined ]
	},
	serialPort: {
		type: "string"
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
	actionSettings: {
		type: "object",
		default: {}
	},
	pluginSettings: {
		type: "object",
		default: {}
	}
}

const Store = require("electron-store");
const store = new Store({ schema });

module.exports = store;