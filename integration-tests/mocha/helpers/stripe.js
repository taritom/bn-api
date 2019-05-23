const supertest = require('supertest');
const pm = require('../test/pm');
const expect = require('chai').expect;
const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest('https://api.stripe.com');
const apiEndPoint = '/v1/tokens';


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/x-www-form-urlencoded')

        .send(pm.substitute(request_body));
};

const stripePk = pm.environment.get('stripePk');
let requestBody = `email=test%40test.com&validation_type=card&referrer=https%3A%2F%2Fstripe.com%2Fdocs%2Fquickstart&card[number]=4242424242424242&card[exp_month]=12&card[exp_year]=19&card[cvc]=001&card[name]=test%40test.com&key=${stripePk}`;

const getToken = async function () {
    const response = await post( requestBody);
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);
    log(response.status);
    log(responseBody);
    expect(response.status).to.equal(200);
    pm.environment.set("last_credit_card_token", json.id);
    return json.id;
}

module.exports = {
    getToken
};