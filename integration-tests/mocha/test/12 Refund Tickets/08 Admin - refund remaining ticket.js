const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/orders/{{last_cart_id}}/refund';


var response;
var responseBody;


const patch = async function (request_body) {
    return baseUrl
        .patch(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
"items": [{
"order_item_id": "{{order_item_id}}",
"ticket_instance_id": "{{ticket_instance_id2}}"
}]
}`;


describe('Admin - refund remaining ticket', function () {
    before(async function () {
        response = await patch(requestBody);
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


    it("no tickets from other organizations", function () {

        let json = JSON.parse(responseBody);
        expect(json.amount_refunded).to.equal(3110);
    });


});

            
