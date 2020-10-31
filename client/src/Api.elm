port module Api exposing (..)

import Json.Encode as Co
import Json.Decode as De
import Json.Decode exposing (Decoder)

port toApi: Co.Value -> Cmd msg

port fromApi: (De.Value -> msg) -> Sub msg

type alias ApiInvoke msg = 
    {
        apiName: String,
        payload: msg
    }

dispatch: ( b -> Co.Value ) -> ApiInvoke b -> Cmd msg
dispatch encode { apiName, payload }= 
    toApi <| Co.object [
        ( "api_name", Co.string apiName ),
        ( "payload", encode payload )
    ]
    
type alias ApiResponse msg = 
    {
        apiName: String,
        payload: Maybe msg,
        error: Bool
    }

responseDecoder: Decoder (Maybe msg) -> Decoder (ApiResponse msg)
responseDecoder decoder = 
    De.map3 ApiResponse 
        (De.field "api_name" De.string)
        (De.field "payload" decoder )
        (De.field "error" De.bool )

decodeFromApi: Decoder (ApiResponse msg) -> De.Value -> Result De.Error (ApiResponse msg)
decodeFromApi decoder value = 
    De.decodeValue decoder value

toSubscription: (Result De.Error (ApiResponse msg) -> b) -> Decoder (Maybe msg) -> Sub b
toSubscription toMsg decoder =
    (decoder
        |> responseDecoder
        |> decodeFromApi)
        >> toMsg
        |> fromApi