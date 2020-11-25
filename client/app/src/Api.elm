port module Api exposing (..)

import Json.Decode as De
import Json.Encode as Ser

port toApi : Ser.Value -> Cmd msg

port fromApi : (De.Value -> msg) -> Sub msg

type Response
    = OnNothing String
    | OnBody ResponseBody


type alias ResponseBody =
    { api_name : String
    , error : Bool
    , payload : De.Value
    }

dispatch : String -> b -> (b -> Ser.Value) -> Cmd msg
dispatch apiName payload serializer =
    toApi <|
        Ser.object
            [ ( "api_name", Ser.string apiName )
            , ( "payload", serializer payload )
            ]

decodeResponse : De.Decoder Response
decodeResponse =
    De.oneOf
        [ De.map OnNothing De.string
        , De.map OnBody <|
            De.map3
                ResponseBody
                (De.field "api_name" De.string)
                (De.field "error" De.bool)
                (De.field "payload" De.value)
        ]

toSubscription : (ResponseBody -> De.Decoder b) -> (String -> b) -> Sub (Result De.Error b)
toSubscription decoder forNothing =
    let
        decode =
            De.decodeValue decodeResponse
                >> Result.andThen
                    (\response ->
                        case response of
                            OnBody body ->
                                De.decodeValue (decoder body) body.payload

                            OnNothing str ->
                                Ok <| forNothing str
                    )
    in
    fromApi decode
