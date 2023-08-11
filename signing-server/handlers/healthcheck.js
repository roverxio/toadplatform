const healthcheck = (req, res) => {
    res.json({status: 'OK'});
}

module.exports = healthcheck;
