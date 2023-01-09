class Action {
    constructor(name, uuid, plugin, tooltip) {
        this.name = name;
        this.uuid = uuid;
        this.plugin = plugin;
        this.tooltip = tooltip;
    }
}

var keys = {
    0: undefined,
    1: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    2: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    3: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    4: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    5: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    6: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    7: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    8: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip"),
    9: new Action("Example Action", "com.elgato.template.action", "com.elgato.template.sdPlugin", "This is an example tooltip")
}

var allActions = { };
var categories = { };

module.exports = { keys, allActions, categories, Action };