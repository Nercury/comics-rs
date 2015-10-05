require.config({
    "paths": {
        "underscore":   "../compiled/bower.ugly",
        "jquery":       "../compiled/bower.ugly",
    },
    baseUrl: '/js/src'
});

require(["main"]);
