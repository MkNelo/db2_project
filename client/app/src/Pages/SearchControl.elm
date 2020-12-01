module Pages.SearchControl exposing (..)
import Html.Attributes exposing (..)
import Html exposing (..)

type RaceType =
    Practice
    | Race

type Category =
    GT
    | GTPro
    | SP

type alias Control msg =
    { raceType: Maybe msg
    , category: Maybe msg
    , pagination: Maybe msg }

control: Control msg -> Html msg
control { raceType, category, pagination } =
        div [ class "container-fluid d-flex" ]
            [ div [ class "btn-group btn-group-toggle mr-auto"
                  , attribute "data-toggle" "buttons" ]
                  [ label [ class "btn btn-secondary border active"
                          , type_ "button"]
                          [ input [ type_ "radio" ] []
                          , text "Ensayo" ]
                  , label [ class "btn btn-secondary border"
                          , type_ "button"]
                          [ input [ type_ "radio" ] []
                          , text "Carrera" ] ]
            , div [ class "btn-group btn-group-toggle mx-auto"
                  , attribute "data-toggle" "buttons" ]
                  [ label [ class "btn btn-secondary border active"
                          , type_ "button"]
                          [ input [ type_ "radio" ] []
                          , text "SP" ]
                  , label [ class "btn btn-secondary border"
                          , type_ "button"]
                          [ input [ type_ "radio" ] []
                          , text "GT" ]
                  , label [ class "btn btn-secondary border"
                          , type_ "button"]
                          [ input [ type_ "radio" ] []
                          , text "LMGP" ]]
            , div [ class "ml-auto border-0" ]
                  [ input [ class "form-control"
                          , type_ "number"
                          , placeholder "Items per page" ] [] ]]
