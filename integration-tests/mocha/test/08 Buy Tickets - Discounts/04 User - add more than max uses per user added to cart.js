const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/cart';


var response;
var responseBody;


const put = async function (request_body) {
    return baseUrl
        .put(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{discount_user_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{discount_user_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
"items": [{
"ticket_type_id": "{{ga_ticket_type_id}}",
"redemption_code": "{{discount_redemption_code}}",
"quantity":3
}
]}`;


describe('User - add more than max uses per user added to cart', function () {
    before(async function () {
        response = await put(requestBody);
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

    it("should be 422", function () {
        expect(response.status).to.equal(422);
    })


    it("Error code should be max_tickets_per_user_reached", function () {

        let json = JSON.parse(responseBody);
        expect(json.fields.quantity[0].code).to.equal("limit_per_person_exceeded");

    });


});

            
