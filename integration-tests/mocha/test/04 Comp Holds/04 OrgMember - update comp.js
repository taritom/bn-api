const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/comps/{{last_comp_id}}';


var response;
var responseBody;


const patch = async function (request_body) {
    return baseUrl
        .patch(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"name": "Michael Robinson",
	"email": "michael-robinson@tari.com",
	"phone":"222-222-2222",
	"quantity": 5
}`;


describe('OrgMember - update comp', function () {
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


    it("comp should have correct information", function () {

        let json = JSON.parse(responseBody);
        expect(json.name).to.equal("Michael Robinson");
        expect(json.phone).to.equal("222-222-2222");
        expect(json.email).to.equal("michael-robinson@tari.com");
        expect(json.quantity).to.equal(5);
    });


});

            
