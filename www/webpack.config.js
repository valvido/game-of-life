const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  experiments: {
    asyncWebAssembly: true // Enable async WebAssembly loading
},
module: {
    rules: [
        {
            test: /\.wasm$/,
            type: "webassembly/async" // Use async WebAssembly module type
        }
    ]
},
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
};
