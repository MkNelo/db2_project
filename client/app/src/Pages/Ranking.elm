module Pages.Ranking exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Pages.SearchControl

type alias RankingModel =
    { searchYear: Maybe Int
    }

type alias RankingItem =
    { position: Int
    , pilots: List String }

item : RankingItem -> Html msg
item { position, pilots } =
    let
        classPosition = case position of
                            1 -> "first-position"
                            2 -> "second-position"
                            3 -> "third-position"
                            _ -> "nth-position"
    in
    div [ class "col row" ]
        [ div [ class <| "col-1 h5 text-dark d-flex border border-dark " ++ classPosition ]
            [ div [ class "m-auto text-center" ]
                [ text <| String.fromInt position ++ "°" ]
            ]
        , div [ class "col m-3 d-flex flex-column" ]
            [ div [ class "row" ]
                [ img
                    [ class "thumbnail mb-2 border rounded"
                    , src "https://www.w3schools.com/w3images/avatar2.png"
                    ]
                    []
                , div [ class "col container" ]
                    [ div [ class "row h5" ]
                        [ div [ class "col-4 text-right" ]
                            [ text "Nombre:" ]
                        , div [ class "col" ]
                            [ text "holi mundo" ]
                        ]
                    , div [ class "row h5" ]
                        [ div [ class "col-4 text-right" ]
                            [ text "Numero:" ]
                        , div [ class "col" ]
                            [ text "........." ]
                        ]
                    , div [ class "row h5" ]
                        [ div [ class "col-4 text-right" ]
                            [ text "Integrantes:" ]
                        , div [ class "col" ]
                            [ ul []
                                <| List.map (\name -> li [] [ text name ])  pilots
                            ]
                        ]
                    ]
                ]
            , div [ class "row w-100 mt-auto" ]
                [ button [ class "btn btn-outline-primary w-100" ]
                    [ text "Mas Detalles" ]
                ]
            ]
        ]


page : Html msg
page =
    div [ class "mx-2 h-100 container-fluid" ]
        [ div [ class "display-4" ]
            [ text "Ranking" ]
        , p [ class "lead mx-2" ] [ text """Mostrando rankings, posicionamiento en la carrera, desde el año /tal/""" ]
        , hr [ class "bg-light my-2" ] []
        , Pages.SearchControl.control { raceType = Nothing, pagination = Nothing, category = Nothing }
        , div [ class "container-fluid mt-4 mx-3" ]
            [ div [ class "row" ]
                [ item { position = 1, pilots =  [ "Integrante 1", "Integrante 2" ] }
                , item { position = 2, pilots = [ "Integrante 1", "Integrante 2", "Integrante 3" ] }
                ]
            ]
        ]
