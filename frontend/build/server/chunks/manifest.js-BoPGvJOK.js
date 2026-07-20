const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.DuSbjRdM.js",app:"_app/immutable/entry/app.m2HvNcqY.js",imports:["_app/immutable/entry/start.DuSbjRdM.js","_app/immutable/chunks/C4X_VlR2.js","_app/immutable/chunks/CM56JfY_.js","_app/immutable/chunks/CoG_F3Z7.js","_app/immutable/entry/app.m2HvNcqY.js","_app/immutable/chunks/CM56JfY_.js","_app/immutable/chunks/-RJlIDMA.js","_app/immutable/chunks/PRbvBVef.js","_app/immutable/chunks/CoG_F3Z7.js","_app/immutable/chunks/DhbyEs9B.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:true},
		nodes: [
			__memo(() => import('./0-DHfDA-Ve.js')),
			__memo(() => import('./1-CB2tFHoC.js')),
			__memo(() => import('./2-B3edc8Qe.js')),
			__memo(() => import('./3-Cg7REsb_.js')),
			__memo(() => import('./4-WxxZcGdg.js')),
			__memo(() => import('./5-CSwqZpEt.js')),
			__memo(() => import('./6-BfshYop-.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/cards",
				pattern: /^\/cards\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/clusters",
				pattern: /^\/clusters\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/definitions",
				pattern: /^\/definitions\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/publish",
				pattern: /^\/publish\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 6 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();

export { manifest as m };
//# sourceMappingURL=manifest.js-BoPGvJOK.js.map
