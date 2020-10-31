module Theme.Icons
    exposing
        ( arrowLeftCircle
        , settings
        , playCircle
        , flag
        , search
        , menu
        )

import Html exposing (Html)
import Svg exposing (Svg, svg)
import Svg.Attributes exposing (..)
import Svg.Events exposing (onMouseOver)


svgFeatherIcon : String -> String -> Maybe String -> String -> List (Svg msg) -> Html msg
svgFeatherIcon pwidth pheight pcolor className =
    svg
        [ class <| "feather feather-" ++ className
        , fill "none"
        , color <| Maybe.withDefault "#ffffff" pcolor
        , height pheight
        , stroke "currentColor"
        , strokeLinecap "round"
        , strokeLinejoin "round"
        , strokeWidth "2"
        , viewBox "0 0 24 24"
        , width pwidth
        ]


arrowLeftCircle : String -> Html msg
arrowLeftCircle back =
    svgFeatherIcon "24" "24" Nothing "arrow-left-circle"
        [ Svg.circle [ cx "12", cy "12", r "10", fill back ] []
        , Svg.polyline [ points "12 8 8 12 12 16" ] []
        , Svg.line [ x1 "16", y1 "12", x2 "8", y2 "12" ] []
        ]


settings : String -> Html msg
settings back =
    svgFeatherIcon "24" "24" Nothing "settings"
        [Svg.path [ fill back, d "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" ] []
        , Svg.circle [ cx "12", cy "12", r "3", fill "#192a56" ] []
        ]

playCircle : String -> Html msg
playCircle back =
    svgFeatherIcon "150" "150" Nothing "play-circle"
        [ Svg.circle [ cx "12", cy "12", r "10", fill back ] []
        , Svg.polygon [ points "10 8 16 12 10 16 10 8" ] []
        ]

flag : Html msg
flag =
    svgFeatherIcon "24" "24" Nothing "flag"
        [ Svg.path [ d "M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z" ] []
        , Svg.line [ x1 "4", y1 "22", x2 "4", y2 "15" ] []
        ]

search : Html msg
search =
    svgFeatherIcon "30" "30" (Just "#808080") "search"
        [ Svg.circle [ cx "11", cy "11", r "8" ] []
        , Svg.line [ x1 "21", y1 "21", x2 "16.65", y2 "16.65" ] []
        ]
menu : Html msg
menu =
    svgFeatherIcon "32" "32" (Just "#000000") "menu"
        [ Svg.line [ x1 "3", y1 "12", x2 "21", y2 "12" ] []
        , Svg.line [ x1 "3", y1 "6", x2 "21", y2 "6" ] []
        , Svg.line [ x1 "3", y1 "18", x2 "21", y2 "18" ] []
        ]
 