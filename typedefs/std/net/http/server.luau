local server = {}

type StatusCode =
    | "200 OK"
    | "201 Created"
    | "204 No Content"
    | "301 Moved Permanently"
    | "302 Found"
    | "400 Bad Request"
    | "401 Unauthorized"
    | "403 Forbidden"
    | "404 Not Found"
    | "405 Method Not Allowed"
    | "500 Internal Server Error"
    | "502 Bad Gateway"
    | "503 Service Unavailable"

type ResponseType = 
	| "Text"
	| "HTML"
	| "JSON"
	| "XML"
	| "CSS"
	| "JavaScript"
	| "Binary"
	| string

export type ServeResponse = {
	status_code: StatusCode,
	response_type: ResponseType,
	body: string,
	headers: {
		[string]: string,
	}?,
	cookies: {
		[string]: string,
	}?,
	http_version: string?,
	reason_phrase: string?,
	redirect_url: string?
}
	
export type ServeRequest = {
	method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
	path: string,
	headers: {
		[string]: string,
	},
	raw_text: string,
	body: string,
}
	
export type ServeConfig = {
	address: string,
	port: string | number,
	handler: (ServeRequest) -> ServeResponse,
}

function server.serve(config: ServeConfig)
	
end

return server