const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

const mode = process.env.NODE_ENV || 'development';
const devMode = mode !== 'production';

module.exports = {
    entry: {
        bundle: ['./src/bootstrap.js']
    },
    resolve: {
        alias: {
            svelte: path.resolve('node_modules', 'svelte')
        },
        extensions: ['.mjs', '.tsx', '.ts', '.js', '.svelte'],
        mainFields: ['svelte', 'browser', 'module', 'main']
    },

    output: {
        path: path.join(__dirname, 'build'),
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
            {
                test: /\.svelte$/,
                use: {
                    loader: 'svelte-loader',
                    options: {
                        emitCss: true,
                        hotReload: true,
                        preprocess: require('svelte-preprocess')({})
                    }
                },
                exclude: /node_modules/,
            },
            {
                test: /\.css$/,
                use: [
                    {
                        loader: MiniCssExtractPlugin.loader,
                        options: {
                            // hot module reload
                            hmr: devMode,
                            // in case hmr does not work:
                            // reloadAll: true,
                        },
                    },
                    'css-loader',
                ]
            }
        ]
    },
    mode,
    plugins: [
        new MiniCssExtractPlugin({}),
        new CopyWebpackPlugin({
            patterns: [
                { from: 'index.html' },
                { from: 'global.css' },
                { from: 'assets', to: 'assets' },
            ]
        })
    ],
    // devtool: devMode ? 'source-map' : false
    devtool: "source-map",
    // optimization: {
    //     minimize: false
    // }
};
