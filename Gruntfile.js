module.exports = function(grunt) {

    grunt.initConfig({
        cssmin: {
            prod: {
                src: 'public/css/compiled/prod.combined.css',
                dest: 'public/css/compiled/prod.css',
            },
        },
        bower_concat: {
            all: {
                dest: 'public/js/compiled/bower.concat.js',
                exclude: [
                    'requirejs',
                    'almond'
                ]
            },
            requirejs: {
                dest: 'public/js/compiled/require.js',
                include: [
                    'requirejs'
                ]
            },
            almond: {
                dest: 'public/js/compiled/almond.js',
                include: [
                    'almond'
                ]
            }
        },
        requirejs: {
            prod: {
                options: {
                    baseUrl: "./public/js/src/",
                    mainConfigFile: './public/js/config.js',
                    name: "../compiled/almond",
                    out: "./public/js/compiled/amd.js",
                    optimize: "none",
                    inlineText: true,
                    preserveLicenseComments: false,
                    include: ['main'],
                    insertRequire: ['main'],
                    wrap: true,
                }
            }
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
        },
        uglify: {
            bower: {
                options: {
                    mangle: true,
                    compress: true
                },
                files: {
                    'public/js/compiled/bower.ugly.js': 'public/js/compiled/bower.compiled.js'
                }
            },
            prod: {
                options: {
                    mangle: true,
                    compress: true
                },
                files: {
                    'public/js/compiled/prod.js': 'public/js/compiled/amd.compiled.js'
                }
            }
        },
        closureCompiler: {
            options: {
                compilerFile: 'utils/compiler.jar',
                compilerOpts: {
                   compilation_level: 'SIMPLE_OPTIMIZATIONS',
                   externs: ['public/js/compiled/require.js'],
                   //define: ["'goog.DEBUG=false'"],
                   warning_level: 'quiet',
                   //warning_level: 'verbose',
                   jscomp_off: ['checkTypes', 'fileoverviewTags', 'checkVars'],
                   summary_detail_level: 0,
                   //output_wrapper: '"(function(){%output%}).call(this);"'
                },
                execOpts: {
                   maxBuffer: 999999 * 1024
                },
                TieredCompilation: true // will use 'java -server -XX:+TieredCompilation -jar compiler.jar'
            },

            bower: {
                src: 'public/js/compiled/bower.concat.js',
                dest: 'public/js/compiled/bower.compiled.js'
            },

            prod: {
                src: 'public/js/compiled/amd.js',
                dest: 'public/js/compiled/amd.compiled.js'
            }
        }
    });

    require('load-grunt-tasks')(grunt);

    grunt.registerTask('dev', [
        'concat:css-dev',
        'bower_concat:all', // get all the libs out of bower except requirejs
        'bower_concat:requirejs', // get requirejs lib out of bower
        'closureCompiler:bower', // compile bower
        'uglify:bower', // uglify bower
    ]);

    grunt.registerTask('prod', [
        'concat:css-prod',
        'cssmin:prod',
        'bower_concat:all', // get all the libs out of bower except requirejs
        'bower_concat:almond', // get requirejs lib out of bower
        'requirejs:prod', // combine all requirejs files into one
        'closureCompiler:prod', // compile combined files
        'uglify:prod', // uglify combined files
    ]);
};
