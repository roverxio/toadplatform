const express = require('express');
const bodyParser = require('body-parser');

const healthcheckHandler = require('./handlers/healthcheck');
const signMessageHandler = require('./handlers/sign_message');
const {ethers, utils} = require("ethers");
const config = require('./config.json');

const app = express();
const PORT = config.port || 3000;
const path_prefix = config.path_prefix || "/app";
const PROVIDER_URL = config.provider_url || "http://localhost:8545";

const PRIVATE_KEY = process.env.PRIVATE_KEY;
const provider = new ethers.providers.JsonRpcProvider(PROVIDER_URL);
const wallet = new ethers.Wallet(PRIVATE_KEY, provider);


// Middlewares
app.use(bodyParser.json());

// Create a new Router object
const apiRouter = express.Router();

// Define the routes under this router
apiRouter.get('/v1/healthcheck', healthcheckHandler);
apiRouter.post('/v1/sign_message', (req, res) => signMessageHandler(req, res, wallet));

// Use the router with a prefix, in this case '/app'
app.use(path_prefix, apiRouter);

app.listen(PORT, () => {
  console.log(`Server started on http://localhost:${PORT}`);
});
