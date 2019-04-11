const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/organizations/{{last_org_id}}/settlements';


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

let requestBody = ``;


describe('Admin - List Weekly Settlements', function () {
    before(async function () {
        response = await get(requestBody);
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
    it("should not be empty", function () {
        let json = JSON.parse(responseBody);
        expect(json.data.length).to.be.greaterThan(0);
// 	for(let i=0; i< json.length; i++){
//     	expect(json[i]).to.have.all.keys('event_id','sales', 'ticket_fees', 'other_fees');
//     	expect(json[i].sales[i]).to.not.be.empty;
//     	expect(json[i].ticket_fees).to.not.be.empty;
//     	expect(json[i].other_fees).to.not.be.empty;
// 	}

    });


});

            
