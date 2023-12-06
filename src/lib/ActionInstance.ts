type ActionInstance = {
	action: Action,
	context: ActionContext,
	states: ActionState[],
	current_state: number,
	settings: any
};
