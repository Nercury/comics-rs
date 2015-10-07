module.exports = function(grunt) {

    grunt.initConfig({
        cssmin: {
            prod: {
                src: 'public/css/compiled/prod.combined.css',
                dest: 'public/css/compiled/prod.css',
            },
        },
        concat: {
            options: {
                separator: '',
            },
            'css-dev': {
                src: [
                    'public/css/reset.css'
                ],
                dest: 'public/css/compiled/dev.css',
            },
            'css-prod': {
                src: [
                    'public/css/reset.css',
                    'public/css/style.css',
                    'public/css/comics.css',
                ],
                dest: 'public/css/compiled/prod.combined.css',
            }
        }
    });

    require('load-grunt-tasks')(grunt);

    grunt.registerTask('dev', [
        'concat:css-dev'
    ]);

    grunt.registerTask('prod', [
        'concat:css-prod',
        'cssmin:prod'
    ]);
};
