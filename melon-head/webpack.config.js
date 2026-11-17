const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const config = require('./config.json');

module.exports = (env, argv) => {
    const mode = argv.mode === 'production' ? 'production' : 'development';
    const apiHost = env['api-host'] ? env['api-host'] : 'http://127.0.0.1:8000';

    console.log("webpack runs in", mode, "mode");

    return {
        mode,
        entry: path.resolve(__dirname, './src/index.ts'),
        module: {
            rules: [
                {
                    test: /\.mjml$/i,
                    use: 'raw-loader',
                },
                {
                    test: /\.scss$/i,
                    use: [
                        "style-loader",
                        {
                            loader: "css-loader",
                            options: {
                                importLoaders: 1,
                                modules: {
                                    namedExport: true,
                                    localIdentName: "[local]--[hash:base64:16]",
                                    exportLocalsConvention: "camelCaseOnly",
                                },
                            },
                        },
                        "sass-loader",
                    ],
                },
                {
                    test: /\.css$/i,
                    use: [
                        "style-loader", "css-loader"
                    ],
                },
                {
                    test: /\.ts$/,
                    use: "ts-loader",
                    exclude: /node_modules/,
                },
                {
                    test: /\.(png|jpg|gif|svg)$/,
                    use: [{
                        loader: 'file-loader',
                        options: {
                            name: '[name]--[contenthash].[ext]',
                            esModule: false,
                        }
                    }]
                },
            ],
        },
        resolve: {
            extensions: ['.js', '.bs.js', '.ts'],
        },
        // react-datepicker 9.1.0 loads optional date-fns-tz via require(variable); we don't use time zones.
        // Pinned to that exact file and line, so any other warning (or this one after an upgrade) still shows.
        ignoreWarnings: [
            (warning) =>
                /node_modules\/react-datepicker\/dist\/index\.es\.js$/.test(warning.module?.resource ?? '') &&
                warning.message === 'Critical dependency: the request of a dependency is an expression' &&
                warning.loc?.start?.line === 138,
        ],
        output: {
            filename: 'bundle-[contenthash].js',
            path: path.resolve(__dirname, 'dist'),
        },
        devServer: {
            port: 1234,
            host: "0.0.0.0",
            historyApiFallback: true,
            proxy: {
                '/api': {
                    target: apiHost,
                    pathRewrite: path => path.replace(/^\/api/g,""),
                    secure: false,
                    changeOrigin: true,
                },
            },
        },
        plugins: [
            new CleanWebpackPlugin(),
            new HtmlWebpackPlugin({
                template: path.resolve(__dirname, "./src/index.html"),
                publicPath: '/',
                favicon: "./static/svg/melon-head.svg",
            }),
        ],
    };
};
