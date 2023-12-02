type Action = {
	name: string,
	uuid: string,
	plugin: string,
	tooltip: string,
	icon: string,
	visible_in_action_list: boolean,
	property_inspector: string,
	controllers: string[],
	states: ActionState[]
};
