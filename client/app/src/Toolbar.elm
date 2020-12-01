module Toolbar exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput, onSubmit)
import Icons as Icons
import Navbar exposing (toggler)


type alias Form msg =
    { inputChange : String -> msg
    , formSubmit : String -> msg
    , inputText : String
    , title: String
    }


toolbar : Form msg -> Html msg
toolbar { inputChange, formSubmit, inputText, title } =
    nav
        [ id "toolbar"
        , class "navbar navbar-dark bg-primary w-100"
        ]
        [ div [ class "navbar-brand" ]
            [ toggler "navbar-toggler mx-2 btn" [ Icons.menu ]
            , text title
            ]
        , Html.form
            [ id "form-year-search"
            , class "form-inline"
            , novalidate True
            , onInput inputChange
            , onSubmit <| formSubmit inputText
            ]
            [ div [ class "form-group"
                  , id "form-year-group"]
                [ div [ class "input-group input-group-lg mt-3" ]
                    [ input
                        [ class "form-control"
                        , placeholder "Año, e.j 1970"
                        , pattern "197[0-9]"
                        , Html.Attributes.form "form-year-search"
                        , value inputText
                        ]
                        []
                    , div [ class "input-group-append" ]
                        [ button
                            [ class "btn btn-dark"
                            , Html.Attributes.form "form-year-search"
                            , type_ "submit"
                            ]
                            [ Icons.search ]
                        ]
                    ]
                ]
            ]
        ]
