const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const config = require('./config.json');

module.exports = (env, argv) => {
    const mode = argv.model === 'production' ? 'production' : 'development';
    const apiHost = env['api-host'] ? env['spi-host'] : 'http://127.0.0.1:8000';

    console.log("webpack runs in", mode, "mode");

    return {
        mode,
        entry: path.resolve(__dirname, './src/index.js'),
        module: {
            rules: [
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
            extensions: ['.js', '.bs.js'],
        },
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
            }),
        ],
    };
};
