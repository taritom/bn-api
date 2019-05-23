const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm');const debug=require('debug');var log = debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/organizations/{{last_org_id}}/settlements/prepare';


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
	"start_utc": "2018-01-01T00:00:00",
	"end_utc": "2030-01-01T00:00:00",
	"comment": "My Comment",
	"only_finished_events": true
}`;


describe('Admin - Pending Weekly Settlement', function () {
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
    });

    after(async function () {
        // add after methods


    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })
// it("should not be empty", function() {
// 	let json = JSON.parse(responseBody);
// 	expect(json.length).to.be.greaterThan(0);
// 	for(let i=0; i< json.length; i++){
//     	expect(json[i]).to.have.all.keys('event_id','sales', 'ticket_fees', 'other_fees');
//     	expect(json[i].sales[i]).to.not.be.empty;
//     	expect(json[i].ticket_fees).to.not.be.empty;
//     	expect(json[i].other_fees).to.not.be.empty;
// 	}

// });


});

            
