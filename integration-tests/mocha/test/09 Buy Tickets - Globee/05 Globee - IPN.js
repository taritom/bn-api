const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/ipns/globee';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))


        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
    "id": "ZbWyAR2VwO4a3BYN063nJ7",
    "status": "paid",
    "total": "603.00",
    "adjusted_total": "603.00",
    "currency": "USD",
    "custom_payment_id": "{{last_cart_id}}",
    "custom_store_reference": null,
    "callback_data": null,
    "customer": {
        "name": null,
        "email": "mike1547729521787@tari.com"
    },
    "payment_details": {
        "currency": "BTC",
        "received_amount": "603.00",
        "received_difference": "0"
    },
    "redirect_url": "https://test.globee.com/payment-request/ZbWyAR2VwO4a3BYN063nJ7",
    "success_url": null,
    "cancel_url": null,
    "ipn_url": "https://f7dac315.ngrok.io/ipns/globee",
    "notification_email": null,
    "confirmation_speed": "medium",
    "expires_at": "2019-01-17 13:07:13",
    "created_at": "2019-01-17 12:52:13"
}`;


describe('Globee - IPN', function () {
    before(async function () {
        response = await post(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
    });

    after(async function () {
        // add after methods


    });


    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


// no tests


});

            
