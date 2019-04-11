const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/orders/{{last_box_office_order_id}}';


var response;
var responseBody;


const patch = async function (request_body) {
    return baseUrl
        .patch(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_boxoffice_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_boxoffice_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{

	"note" : "Client to collect after 5pm"
}`;

let json = {};

describe('Box Office - Update order note', function () {
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

        json = JSON.parse(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("Note should be updated", function () {
        expect(json.note).to.equal("Client to collect after 5pm");
    })


});

            
