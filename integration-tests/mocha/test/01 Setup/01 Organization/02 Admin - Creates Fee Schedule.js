const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm');const debug=require('debug');var log = debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/organizations/{{last_org_id}}/fee_schedule';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
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
"name": "Fee_schedule_{{$timestamp}}",
"ranges": [
	{
		"min_price_in_cents": 1,
		"company_fee_in_cents": 4,
		"client_fee_in_cents": 6
	}
]
}`;


describe('Admin - Creates Fee Schedule', function () {
    var r;

    before(async function () {
        response = await post(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        log(responseBody);


        r = JSON.parse(responseBody);
    });


    after(async function () {
        // add after methods

        pm.environment.set("last_fee_schedule_id", JSON.parse(responseBody).id);

    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })


    it("should be one result", function () {
        expect(r.ranges.length).to.equal(1);
    });

    it("should have correct min_price_in_cents", function () {
        expect(r.ranges[0].min_price_in_cents).to.equal(1);
    });

    it("should have correct fee_in_cents", function () {
        expect(r.ranges[0].fee_in_cents).to.equal(10);
    });


});

            
