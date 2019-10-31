const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm');const debug=require('debug');var log = debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/artists/{{last_artist_id}}';


var response;
var responseBody;


const put = async function (request_body) {
    return baseUrl
        .put(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{token}}'))

        .send(pm.substitute(request_body));
};

let requestBody = `{
	"name":"Artist updated {{$timestamp}}",
	"bio": "Artist bio updated",
	"main_genre": "blues"
}`;
let json = {};

describe('Admin - Update Artist', function () {
    before(async function () {
        response = await put(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        log(responseBody);


        json = JSON.parse(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);

    })


    it("should be updated", function () {
        expect(json.name).to.include("Artist updated");
        expect(json.bio).to.equal("Artist bio updated");
        expect(json.main_genre_id).to.not.be.null;
        expect(json.main_genre).to.equal("blues");
    });


});


