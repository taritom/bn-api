const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm');
const user = require('../../../helpers/user');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/tickets/receive';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{user2_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{user2_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
    "transfer_key": "{{transfer_transfer_key}}",
    "sender_user_id": "{{transfer_sender_user_id}}",
    "num_tickets": {{transfer_num_tickets}},
    "signature": "{{transfer_signature}}"
}`;


describe('User2 - Tickets - Receive Transfer', function () {
    before(async function () {
        await user.registerAndLogin("user2_token");
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


});

            
