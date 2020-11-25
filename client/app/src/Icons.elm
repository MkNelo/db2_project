module Icons
    exposing
        ( award
        , menu
        , playCircle
        , clock
        , search
        , home
        )

import Html exposing (Html)
import Svg exposing (Svg, svg)
import Svg.Attributes exposing (..)


svgFeatherIcon : String -> String -> String -> List (Svg msg) -> Html msg
svgFeatherIcon h w className =
    svg
        [ class <| "feather feather-" ++ className
        , fill "none"
        , height h
        , stroke "currentColor"
        , strokeLinecap "round"
        , strokeLinejoin "round"
        , strokeWidth "2"
        , viewBox "0 0 24 24"
        , width w
        ]


award : Html msg
award =
    svgFeatherIcon "24" "24" "award"
        [ Svg.circle [ cx "12", cy "8", r "7" ] []
        , Svg.polyline [ points "8.21 13.89 7 23 12 20 17 23 15.79 13.88" ] []
        ]


menu : Html msg
menu =
    svgFeatherIcon "24" "24" "menu"
        [ Svg.line [ x1 "3", y1 "12", x2 "21", y2 "12" ] []
        , Svg.line [ x1 "3", y1 "6", x2 "21", y2 "6" ] []
        , Svg.line [ x1 "3", y1 "18", x2 "21", y2 "18" ] []
        ]

playCircle : String -> Html msg
playCircle class =
    svgFeatherIcon "96" "96" ("play-circle " ++ class)
        [ Svg.circle [ cx "12", cy "12", r "10" ] []
        , Svg.polygon [ points "10 8 16 12 10 16 10 8" ] []
        ]

clock : Html msg
clock =
    svgFeatherIcon "24" "24" "clock"
        [ Svg.circle [ cx "12", cy "12", r "10" ] []
        , Svg.polyline [ points "12 6 12 12 16 14" ] []
        ] 

search : Html msg
search =
    svgFeatherIcon "24" "24" "search"
        [ Svg.circle [ cx "11", cy "11", r "8" ] []
        , Svg.line [ x1 "21", y1 "21", x2 "16.65", y2 "16.65" ] []
        ]

home : Html msg
home =
    svgFeatherIcon "24" "24" "home"
        [ Svg.path [ d "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" ] []
        , Svg.polyline [ points "9 22 9 12 15 12 15 22" ] []
        ]
