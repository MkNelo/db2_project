module Api.Report exposing (report, report2, report3, report4, report5, reportSubscription)

import Api
import Json.Decode as De
import Json.Encode as Ser

type alias QueryInfo =
    { name : String
    , params : Ser.Value
    }


decoder : QueryInfo -> Ser.Value
decoder { name, params } =
    Ser.object
        [ ( "name", Ser.string name )
        , ( "payload", params )
        ]


dispatchReport : String -> De.Value -> Cmd msg
dispatchReport name params =
    let
        queryInfo =
            QueryInfo name params
    in
    Api.dispatch "app/report" queryInfo decoder

report: String -> ( a, a -> Ser.Value ) -> Cmd msg
report name ( p1, e1 ) =
    let
        params =
            Ser.list
                (\x -> x)
                [ e1 p1 ]
    in
    dispatchReport name params

report2: String -> ( a, a -> Ser.Value ) -> ( b, b -> Ser.Value ) -> Cmd msg
report2 name (p1, e1) (p2, e2) =
    let
        params =
            Ser.list
                (\x -> x)
                [ e1 p1
                , e2 p2 ]
    in
    dispatchReport name params

report3: String -> ( a, a -> Ser.Value ) -> ( b, b -> Ser.Value ) -> ( c, c -> Ser.Value ) -> Cmd msg
report3 name (p1, e1) (p2, e2) (p3, e3) =
    let
        params =
            Ser.list
                (\x -> x)
                [ e1 p1
                , e2 p2
                , e3 p3 ]
    in
    dispatchReport name params

report4: String -> ( a, a -> Ser.Value ) -> ( b, b -> Ser.Value ) -> ( c, c -> Ser.Value ) -> ( d, d -> Ser.Value ) -> Cmd msg
report4 name (p1, e1) (p2, e2) (p3, e3) (p4, e4) =
    let
        params =
            Ser.list
                (\x -> x)
                [ e1 p1
                , e2 p2
                , e3 p3
                , e4 p4 ]
    in
    dispatchReport name params

report5: String -> ( a, a -> Ser.Value ) -> ( b, b -> Ser.Value ) -> ( c, c -> Ser.Value ) -> ( d, d -> Ser.Value ) -> ( e, e -> Ser.Value )-> Cmd msg
report5 name (p1, e1) (p2, e2) (p3, e3) (p4, e4) (p5, e5) =
    let
        params =
            Ser.list
                (\x -> x)
                [ e1 p1
                , e2 p2
                , e3 p3
                , e4 p4
                , e5 p5 ]
    in
    dispatchReport name params

reportSubscription : De.Decoder b -> Sub (Result String b)
reportSubscription decodeB =
    Api.toSubscription
        (\response ->
            if response.error then
                De.map Err De.string

            else
                De.map Ok decodeB
        )
        Err
        |> Sub.map
            (\result ->
                case result of
                    Ok res ->
                        res

                    Err ms ->
                        Err <| De.errorToString ms
            )
